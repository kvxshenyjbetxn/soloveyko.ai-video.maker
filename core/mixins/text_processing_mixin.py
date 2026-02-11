import re
import unicodedata
from typing import List, Dict, Any, Tuple, Optional
from utils.logger import logger, LogLevel


class TextProcessingMixin:
    """
    Mixin for TaskProcessor to handle text segmentation and subtitle alignment.
    Supports multiple languages including CJK, Arabic, Cyrillic, and Latin scripts.
    """
    
    # ==================== TEXT SEGMENTATION ====================
    
    def split_text_into_chunks(self, text: str, num_chunks: int, special_count: int = 0) -> List[str]:
        """
        Splits text into chunks. 
        If special_count > 0, the first 'special_count' chunks will be single sentences (shorter),
        and the rest of the text will be distributed among (num_chunks - special_count) chunks.
        
        Args:
            text: The text to split.
            num_chunks: Target number of chunks.
            special_count: Number of chunks at the start to treat specially (video mode).
            
        Returns:
            List of text chunks.
        """
        if not text or not text.strip():
            return []
        
        if num_chunks <= 1:
            return [text.strip()]

        # Split into sentences using multi-language punctuation
        sentences = self._split_into_sentences(text)
        
        if not sentences:
            return [text.strip()]
        
        if len(sentences) <= num_chunks:
            # If we have fewer sentences than desired chunks, return sentences as-is
            return sentences
            
        chunks = []
        remaining_sentences = sentences
        
        # --- PHASE 1: Special Chunks (Video Mode) ---
        if special_count > 0:
            count = min(special_count, len(sentences))
            # Take first 'count' sentences as individual chunks
            for i in range(count):
                chunks.append(remaining_sentences[i])
            
            # Remove used sentences
            remaining_sentences = remaining_sentences[count:]
            
            logger.log(f"[TextProcessing] Special split: Created {count} short segments. Remaining sentences: {len(remaining_sentences)}", level=LogLevel.DEBUG)
        
        if not remaining_sentences:
            return chunks

        # --- PHASE 2: Even Distribution of Remaining Text ---
        # Calculate how many chunks we have left to fill
        # We must ensure we don't exceed num_chunks if possible, but also don't clump everything if num_chunks is used up
        
        remaining_slots = max(1, num_chunks - len(chunks))
        
        # Calculate target length per chunk for remaining text
        total_len = sum(len(s) for s in remaining_sentences)
        target_chunk_len = total_len / remaining_slots
        
        current_chunk = []
        current_len = 0
        
        # Track how many chunks we've created in this phase
        chunks_created = 0
        
        for i, sentence in enumerate(remaining_sentences):
            sentence_len = len(sentence)
            
            # If we are filling the LAST slot, we must put everything remaining into it
            if chunks_created >= remaining_slots - 1:
                current_chunk.append(sentence)
                continue

            # Otherwise, check if adding this sentence makes us closer to target
            # or if we exceed it significantly
            will_exceed = (current_len + sentence_len) > target_chunk_len
            
            # Heuristic: if current is empty, take it.
            if not current_chunk:
                current_chunk.append(sentence)
                current_len += sentence_len
                continue
            
            # If adding this makes us exceed target significantly, split NOW
            if will_exceed and (current_len + sentence_len) > (target_chunk_len * 1.1):
                 chunks.append(" ".join(current_chunk))
                 chunks_created += 1
                 current_chunk = [sentence]
                 current_len = sentence_len
                 continue
            
            # If adding this keeps us under or close to target, add it
            current_chunk.append(sentence)
            current_len += sentence_len
        
        # Add the final accumulated chunk
        if current_chunk:
            chunks.append(" ".join(current_chunk))
        
        logger.log(f"[TextProcessing] Split text into {len(chunks)} chunks (requested: {num_chunks}, special: {special_count})", level=LogLevel.INFO)
        return chunks
    
    def _split_into_sentences(self, text: str) -> List[str]:
        """
        Splits text into sentences supporting multiple languages.
        
        Supports:
        - Latin/Cyrillic: . ! ?
        - Chinese/Japanese: 。！？
        - Arabic: ؟ (question mark)
        - Thai: ฯ (abbreviation mark, sometimes used as period)
        - Devanagari: । (danda)
        """
        # Comprehensive sentence-ending punctuation
        # Using lookbehind to keep punctuation with the sentence
        # Pattern: split after these marks followed by whitespace or end of string
        sentence_enders = r'[.!?。！？؟।॥\u0964\u0965]'
        
        # Split but keep the delimiter with the preceding text
        # We use a capturing group to include the delimiter in the result
        parts = re.split(f'({sentence_enders}+)', text)
        
        sentences = []
        current = ""
        
        for i, part in enumerate(parts):
            if not part:
                continue
            
            # Check if this part is just punctuation
            if re.match(f'^{sentence_enders}+$', part):
                current += part
            else:
                # If we have accumulated text, save it
                if current.strip():
                    sentences.append(current.strip())
                current = part
        
        # Don't forget the last piece
        if current.strip():
            sentences.append(current.strip())
        
        return sentences
    
    # ==================== SUBTITLE ALIGNMENT ====================
    
    def align_segments_to_subtitles(
        self, 
        segments: List[str], 
        subtitles_path: str,
        task_id: str = ""
    ) -> List[Dict[str, Any]]:
        """
        Aligns text segments to subtitle timings using an 'Anchor & Interpolate' strategy.
        This ensures that a single missing segment does not break synchronization for subsequent segments.
        """
        import os
        
        prefix = f"[{task_id}] " if task_id else ""
        
        if not os.path.exists(subtitles_path):
            logger.log(f"{prefix}[Sync] Subtitle file not found: {subtitles_path}", level=LogLevel.ERROR)
            return []
        
        # Parse subtitles
        if subtitles_path.lower().endswith('.ass'):
            subs = self._parse_ass_events(subtitles_path)
        else:
            subs = self._parse_srt_file(subtitles_path)
        
        if not subs:
            logger.log(f"{prefix}[Sync] No subtitle events found", level=LogLevel.WARNING)
            return []
        
        # Build text stream
        text_stream, char_to_time = self._build_text_stream(subs)
        if not text_stream: return []
        
        # CRITICAL FIX: Normalize while keeping map to original indices
        # This ensures that when we find a match in normalized text (shorter),
        # we can look up the correct time in the original text (longer).
        stream_normalized, stream_map = self._normalize_text_with_mapping(text_stream)
        
        # --- PHASE 1: IDENTIFY ANCHORS ---
        # Find matches, but ONLY accept them as anchors if confidence is high (> 65%)
        matches = [None] * len(segments)
        last_search_start = 0
        ANCHOR_THRESHOLD = 0.65
        
        for i, segment in enumerate(segments):
            seg_normalized = self._normalize_text(segment)
            if not seg_normalized: continue
            
            # Start searching from the end of the last KNOWN STRONG ANCHOR
            match = self._find_segment_in_stream(
                seg_normalized, 
                stream_normalized, 
                start_from=last_search_start
            )
            
            if match:
                norm_start_char, norm_end_char, confidence = match
                
                # CRITICAL FIX: Map normalized indices back to original indices
                # handle out of bounds (though unlikely if specific logic is correct)
                if norm_start_char >= len(stream_map): norm_start_char = len(stream_map) - 1
                if norm_end_char >= len(stream_map): norm_end_char = len(stream_map) - 1
                
                orig_start_char = stream_map[norm_start_char]
                orig_end_char = stream_map[norm_end_char]
                
                if confidence < ANCHOR_THRESHOLD:
                    logger.log(f"{prefix}[Sync] Seg {i+1} weak match: {confidence:.0%} (Ignored as anchor)", level=LogLevel.WARNING)
                    continue
                
                start_time = self._char_to_time(orig_start_char, char_to_time, 'start')
                end_time = self._char_to_time(orig_end_char, char_to_time, 'end')
                
                # Sanity: Ensure positive duration
                if end_time <= start_time: end_time = start_time + 0.5
                
                matches[i] = {
                    "index": i,
                    "start": start_time,
                    "end": end_time,
                    "duration": end_time - start_time,
                    "confidence": confidence,
                    "char_end": norm_end_char # Use normalized index for search cursor!
                }
                
                # Update search cursor ONLY for strong anchors (use normalized index)
                last_search_start = norm_end_char
            else:
                pass

        # --- PHASE 2: INTERPOLATION (GAP FILLING) ---
        final_timings = []
        
        # Track the valid end time of the previous processed block
        # Start at 0.0 or the start of the first subtitle
        prev_valid_end = subs[0]['start'] if subs else 0.0
        
        i = 0
        while i < len(segments):
            if matches[i]:
                # === CASE: VALID MATCH ===
                current = matches[i]
                
                # Correction: Check if this match overlaps with previous end
                # (Can happen if normalization matches overlapped text or tight timings)
                if current['start'] < prev_valid_end:
                    current['start'] = prev_valid_end
                    if current['end'] <= current['start']:
                        current['end'] = current['start'] + max(0.5, current['duration'])
                    current['duration'] = current['end'] - current['start']
                
                final_timings.append(current)
                prev_valid_end = current['end']
                
                logger.log(f"{prefix}[Sync] Seg {i+1} matched: {current['start']:.2f}-{current['end']:.2f} (Conf: {current['confidence']:.0%})", level=LogLevel.DEBUG)
                i += 1
            else:
                # === CASE: GAP DETECTED ===
                gap_start_idx = i
                
                # Find how long this gap is (how many consecutive missing segments)
                gap_end_idx = i
                while gap_end_idx < len(segments) and matches[gap_end_idx] is None:
                    gap_end_idx += 1
                
                # The gap is segments [gap_start_idx ... gap_end_idx-1]
                num_missing = gap_end_idx - gap_start_idx
                
                # Identify the Time Boundary for this gap
                # Start: prev_valid_end (End of last anchor)
                # End: Start of NEXT anchor (or end of subtitles if no more anchors)
                
                next_valid_start = 0.0
                if gap_end_idx < len(segments):
                    # We found a future anchor
                    next_valid_start = matches[gap_end_idx]['start']
                else:
                    # No more anchors, use total duration of subtitles
                    total_dur = subs[-1]['end'] if subs else (prev_valid_end + num_missing * 5.0)
                    next_valid_start = total_dur
                
                # Calculate Time Budget
                time_budget = max(0.0, next_valid_start - prev_valid_end)
                
                logger.log(f"{prefix}[Sync] Gap detected: Segments {gap_start_idx+1}-{gap_end_idx}. Budget: {time_budget:.2f}s (From {prev_valid_end:.2f} to {next_valid_start:.2f})", level=LogLevel.WARNING)
                
                # Heuristic: Distribute budget based on text length
                missing_segments_text = [segments[k] for k in range(gap_start_idx, gap_end_idx)]
                total_text_len = sum(len(self._normalize_text(s)) for s in missing_segments_text) or 1
                
                current_time_cursor = prev_valid_end
                
                for k in range(gap_start_idx, gap_end_idx):
                    seg_text = self._normalize_text(segments[k])
                    seg_len = len(seg_text) if seg_text else 1
                    
                    weight = seg_len / total_text_len
                    allocated_dur = time_budget * weight
                    
                    # Log if we are squeezing too tight
                    if allocated_dur < 0.2:
                        logger.log(f"{prefix}[Sync] Warning: Very short duration allocated for seg {k+1}: {allocated_dur:.2f}s", level=LogLevel.DEBUG)
                    
                    timing = {
                        "index": k,
                        "start": current_time_cursor,
                        "end": current_time_cursor + allocated_dur,
                        "duration": allocated_dur,
                        "confidence": 0.0 # Interpolated
                    }
                    final_timings.append(timing)
                    current_time_cursor += allocated_dur
                
                # Update boundary for next loop
                prev_valid_end = current_time_cursor
                
                # Advance iterator past the gap
                i = gap_end_idx

        logger.log(
            f"{prefix}[Sync] Aligned {len(final_timings)} segments. "
            f"Anchors found: {sum(1 for m in matches if m is not None)}/{len(segments)}",
            level=LogLevel.INFO
        )
        
        return final_timings
    
    def _build_text_stream(self, subs: List[Dict]) -> Tuple[str, List[Dict]]:
        """
        Builds a continuous text stream from subtitles with character-to-time mapping.
        
        Returns:
            (text_stream, char_to_time_map)
            
            char_to_time_map is a list of dicts:
            [{"char_start": 0, "char_end": 15, "time_start": 0.0, "time_end": 2.5}, ...]
        """
        text_parts = []
        char_map = []
        current_char = 0
        
        for sub in subs:
            sub_text = sub.get('text', '').strip()
            if not sub_text:
                continue
            
            # Add space between subtitles for cleaner matching
            if text_parts:
                text_parts.append(' ')
                current_char += 1
            
            start_char = current_char
            text_parts.append(sub_text)
            current_char += len(sub_text)
            end_char = current_char
            
            char_map.append({
                'char_start': start_char,
                'char_end': end_char,
                'time_start': sub['start'],
                'time_end': sub['end']
            })
        
        return ''.join(text_parts), char_map
    
    def _find_segment_in_stream(
        self, 
        segment: str, 
        stream: str, 
        start_from: int = 0
    ) -> Optional[Tuple[int, int, float]]:
        """
        Finds the best match for a segment in the text stream using difflib.
        Uses longest common substring to find anchor, then expands to check full segment.
        """
        if not segment or not stream:
            return None
        
        # Limit search area
        search_area = stream[start_from:]
        if not search_area:
            return None
            
        import difflib
        
        # Thresholds
        # For very short segments, we require high confidence
        base_threshold = 0.5
        if len(segment) < 20:
             base_threshold = 0.65
        
        # 1. Exact Match (Fastest)
        pos = search_area.find(segment)
        if pos != -1:
            return (start_from + pos, start_from + pos + len(segment), 1.0)
            
        # 2. Fuzzy Match (Smart)
        matcher = difflib.SequenceMatcher(None, segment, search_area)
        
        # Find the best "anchor" (longest common substring)
        # This is efficient and robust to shifts/garbled text ends
        match = matcher.find_longest_match(0, len(segment), 0, len(search_area))
        
        # match.a = start in segment
        # match.b = start in search_area
        # match.size = length of match
        
        if match.size == 0:
            return None
            
        # Validation: Anchor must be significant
        # either cover >30% of segment, or be >15 chars long.
        coverage = match.size / len(segment)
        if coverage < 0.3 and match.size < 15:
            return None
            
        # Extrapolate full segment boundaries
        # We know where the match is in the segment (match.a) and in search_area (match.b)
        
        rel_start = match.b - match.a
        rel_end = rel_start + len(segment)
        
        # Clamp to bounds
        rel_start = max(0, rel_start)
        rel_end = min(len(search_area), rel_end)
        
        # Extract candidate text
        candidate = search_area[rel_start:rel_end]
        
        # Calculate full similarity ratio
        confidence = difflib.SequenceMatcher(None, segment, candidate).ratio()
        
        if confidence >= base_threshold:
            return (start_from + rel_start, start_from + rel_end, confidence)
            
        return None
    
    def _fuzzy_find(
        self, 
        needle: str, 
        haystack: str, 
        threshold: float = 0.6
    ) -> Optional[Tuple[int, float]]:
        """
        Finds the best fuzzy match for needle in haystack.
        
        Returns:
            (position, confidence) or None if no match above threshold.
        """
        if not needle or not haystack:
            return None
        
        needle_len = len(needle)
        best_pos = -1
        best_score = 0.0
        
        # Sliding window approach
        for i in range(len(haystack) - needle_len + 1):
            window = haystack[i:i + needle_len]
            
            # Quick check: if first and last chars match, compute full score
            if window[0] == needle[0] or window[-1] == needle[-1]:
                score = self._similarity(needle, window)
                if score > best_score:
                    best_score = score
                    best_pos = i
                    
                    # Early exit if perfect match
                    if score == 1.0:
                        return (best_pos, 1.0)
        
        if best_score >= threshold:
            return (best_pos, best_score)
        
        return None
    
        import difflib
        return difflib.SequenceMatcher(None, s1, s2).ratio()
    
    def _char_to_time(
        self, 
        char_pos: int, 
        char_map: List[Dict], 
        mode: str = 'start'
    ) -> float:
        """
        Maps a character position to a time value.
        
        Args:
            char_pos: Character position in the stream.
            char_map: Character-to-time mapping from _build_text_stream.
            mode: 'start' or 'end' - which boundary to prefer.
            
        Returns:
            Time in seconds.
        """
        if not char_map:
            return 0.0
        
        # Find the subtitle segment containing this character
        for entry in char_map:
            if entry['char_start'] <= char_pos < entry['char_end']:
                # Interpolate within the segment
                segment_len = entry['char_end'] - entry['char_start']
                segment_duration = entry['time_end'] - entry['time_start']
                
                if segment_len > 0:
                    ratio = (char_pos - entry['char_start']) / segment_len
                    return entry['time_start'] + ratio * segment_duration
                else:
                    return entry['time_start'] if mode == 'start' else entry['time_end']
        
        # If past all segments, return end of last
        if char_pos >= char_map[-1]['char_end']:
            return char_map[-1]['time_end']
        
        # Before all segments
        return char_map[0]['time_start']
    
    def _fix_timing_overlaps(self, timings: List[Dict]) -> List[Dict]:
        """
        Ensures timings are sequential without overlaps or gaps.
        - Fixes overlaps by adjusting start times
        - Closes gaps by extending previous segment's end time
        """
        if not timings:
            return timings
        
        fixed = [timings[0].copy()]
        
        for i in range(1, len(timings)):
            current = timings[i].copy()
            prev = fixed[-1]
            
            # Case 1: Overlap - current starts before previous ends
            if current['start'] < prev['end']:
                current['start'] = prev['end']
            
            # Case 2: Gap - current starts after previous ends
            elif current['start'] > prev['end'] + 0.1:  # 0.1s tolerance for small gaps
                gap = current['start'] - prev['end']
                
                # Only close small gaps (< 5.0s) by extending previous static image.
                # If gap is huge (e.g. lost segment), do NOT stretch the previous image indefinitely.
                if gap < 5.0:
                    prev['end'] = current['start']
                    prev['duration'] = prev['end'] - prev['start']
                else:
                    # For large gaps, just set current start to prev end to maintain continuity visually
                    # BUT update duration to be realistic for the text length
                    # This effectively pulls the current segment back time-wise, desyncing it slightly
                    # but preventing the "frozen image" effect.
                    # BETTER: Leave the gap as is? No, black screen is bad.
                    # Best: Extend prev end halfway, pull current start halfway.
                    # For now: Drag current start back.
                    current['start'] = prev['end']
                    current['duration'] = current['end'] - current['start']
            
            # Ensure valid duration
            if current['end'] <= current['start']:
                current['end'] = current['start'] + 0.5
            
            current['duration'] = current['end'] - current['start']
            fixed.append(current)
        
        return fixed
    
    # ==================== TEXT NORMALIZATION ====================
    
    def _normalize_text(self, text: str) -> str:
        """
        Normalizes text for comparison.
        - Converts to lowercase
        - Removes punctuation
        - Normalizes Unicode
        - Collapses whitespace
        """
        if not text:
            return ""
        
        # Unicode normalization (canonical decomposition + composition)
        text = unicodedata.normalize('NFKC', text)
        
        # Replace hyphens with space to handle "Сахар-то" -> "Сахар то" match
        text = text.replace('-', ' ').replace('—', ' ')
        
        # Remove all punctuation and symbols (Unicode-aware)
        punctuation = (
            '!"#$%&\'()*+,./:;<=>?@[\\]^_`{|}~'  # ASCII (removed -)
            '。！？、，；：""''【】（）…·'  # CJK (removed —)
            '؟،؛'  # Arabic
            '।॥'  # Devanagari
            '「」『』〈〉《》〔〕'  # Brackets
        )
        
        for char in punctuation:
            text = text.replace(char, ' ')
        
        # Lowercase
        text = text.lower()
        
        # Collapse whitespace
        text = ' '.join(text.split())
        
        return text.strip()

    def _normalize_text_with_mapping(self, text: str) -> Tuple[str, List[int]]:
        """
        Normalizes text and returns the valid text plus a mapping 
        from normalized_index -> original_index.
        """
        if not text:
            return "", []
            
        normalized_chars = []
        mapping = []
        
        # Punctuation set (same as in _normalize_text)
        punctuation = set(
            '!"#$%&\'()*+,./:;<=>?@[\\]^_`{|}~'
            '。！？、，；：""''【】（）…·'
            '؟،؛'
            '।॥'
            '「」『』〈〉《》〔〕'
        )
        
        # Use unicodedata for consistency
        # Note: mapping logic is simpler if we iterate original characters
        # But NFKC can change length (e.g. separate accents).
        # For simplicity and robustness given we map char-to-time,
        # we will iterate the ORIGINAL string and decide if we keep the char.
        # This avoids complex NFKC mapping issues, but means strict Unicode composition
        # might be slightly less robust if the subtitle file mixes forms. 
        # Typically subtitle files are consistent.
        
        # Step 1: Lowercase and replace hyphens in ORIGINAL (preserves 1:1)
        # Replacing hyphen with space effectively keeps the char count same (len('-')==1, len(' ')==1)
        # But we must be careful with 'replace'.
        
        # Let's iterate manually.
        
        for i, char in enumerate(text):
            c = char
            
            # Hyphen handling
            if c in ('-', '—'):
                c = ' '
            
            # Punctuation handling
            if c in punctuation:
                c = ' '
                
            # Whitespace handling later - for now turn to space
            if c.isspace():
                c = ' '
                
            # Lowercase
            c = c.lower()
            
            # If it's a space
            if c == ' ':
                # Only add if previous wasn't a space (collapse)
                if normalized_chars and normalized_chars[-1] == ' ':
                    continue
                # If it is leading space, skip
                if not normalized_chars:
                    continue
            
            # Add to result
            normalized_chars.append(c)
            mapping.append(i)
            
        # Trim trailing space
        if normalized_chars and normalized_chars[-1] == ' ':
             normalized_chars.pop()
             mapping.pop()
             
        return "".join(normalized_chars), mapping
    
    # ==================== SUBTITLE PARSING ====================
    
    def _parse_ass_events(self, path: str) -> List[Dict]:
        """
        Parses ASS subtitle file and extracts dialogue events.
        
        Returns:
            List of dicts: [{"start": float, "end": float, "text": str}, ...]
        """
        events = []
        try:
            with open(path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            in_events = False
            format_spec = []
            
            for line in lines:
                line = line.strip()
                
                if line == '[Events]':
                    in_events = True
                    continue
                
                if line.startswith('[') and line.endswith(']') and in_events:
                    # New section started, stop processing events
                    break
                
                if in_events:
                    if line.startswith('Format:'):
                        format_spec = [x.strip() for x in line[7:].split(',')]
                    elif line.startswith('Dialogue:'):
                        # Split into format_spec number of parts
                        # The last field (Text) may contain commas, so limit the split
                        parts = line[9:].split(',', len(format_spec) - 1)
                        
                        if len(parts) == len(format_spec):
                            row = dict(zip(format_spec, parts))
                            
                            # Clean ASS formatting codes from text
                            text = row.get('Text', '')
                            text = self._clean_ass_text(text)
                            
                            if text.strip():
                                events.append({
                                    'start': self._ass_time_to_sec(row.get('Start', '0:00:00.00')),
                                    'end': self._ass_time_to_sec(row.get('End', '0:00:00.00')),
                                    'text': text.strip()
                                })
                                
        except Exception as e:
            logger.log(f"Error parsing ASS file: {e}", level=LogLevel.ERROR)
            return []
        
        return events
    
    def _clean_ass_text(self, text: str) -> str:
        r"""
        Removes ASS formatting codes from text.
        Examples: {\i1}, {\b1}, {\an8}, {\pos(100,200)}, {\c&HFFFFFF&}
        """
        # Remove {...} blocks
        text = re.sub(r'\{[^}]*\}', '', text)
        # Replace \N (ASS line break) with space
        text = text.replace('\\N', ' ').replace('\\n', ' ')
        return text.strip()
    
    def _parse_srt_file(self, path: str) -> List[Dict]:
        """
        Parses SRT subtitle file.
        
        Returns:
            List of dicts: [{"start": float, "end": float, "text": str}, ...]
        """
        events = []
        try:
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Split by double newline (subtitle blocks)
            blocks = re.split(r'\n\s*\n', content.strip())
            
            for block in blocks:
                lines = block.strip().split('\n')
                if len(lines) >= 2:
                    # First line is index (ignore)
                    # Second line is timestamp
                    # Rest is text
                    timestamp_line = lines[1] if len(lines) > 1 else ""
                    text_lines = lines[2:] if len(lines) > 2 else []
                    
                    # Parse timestamp: 00:00:01,234 --> 00:00:04,567
                    match = re.match(
                        r'(\d+):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d+):(\d{2}):(\d{2})[,.](\d{3})',
                        timestamp_line
                    )
                    
                    if match:
                        start = (
                            int(match.group(1)) * 3600 +
                            int(match.group(2)) * 60 +
                            int(match.group(3)) +
                            int(match.group(4)) / 1000
                        )
                        end = (
                            int(match.group(5)) * 3600 +
                            int(match.group(6)) * 60 +
                            int(match.group(7)) +
                            int(match.group(8)) / 1000
                        )
                        
                        text = ' '.join(text_lines).strip()
                        # Remove HTML-style tags
                        text = re.sub(r'<[^>]+>', '', text)
                        
                        if text:
                            events.append({
                                'start': start,
                                'end': end,
                                'text': text
                            })
                            
        except Exception as e:
            logger.log(f"Error parsing SRT file: {e}", level=LogLevel.ERROR)
            return []
        
        return events
    
    def _ass_time_to_sec(self, t_str: str) -> float:
        """
        Converts ASS timestamp format (H:MM:SS.cs) to seconds.
        """
        try:
            parts = t_str.split(':')
            if len(parts) == 3:
                h = int(parts[0])
                m = int(parts[1])
                s = float(parts[2])
                return h * 3600 + m * 60 + s
        except:
            pass
        return 0.0
    
    # ==================== LEGACY COMPATIBILITY ====================
    
    def align_images_to_subtitles(self, segments_map: Dict[int, str], subtitles_path: str) -> List[float]:
        """
        Legacy method for backward compatibility.
        Converts old format to new and returns just durations.
        """
        # Convert old format to new
        segments = [segments_map[i] for i in sorted(segments_map.keys())]
        
        # Use new method
        timings = self.align_segments_to_subtitles(segments, subtitles_path)
        
        # Extract just durations
        return [t['duration'] for t in timings]
