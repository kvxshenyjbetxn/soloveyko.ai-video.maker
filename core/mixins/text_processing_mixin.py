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
            
            # Calculate remaining chunks needed
            remaining_chunks_target = max(1, num_chunks - len(chunks))
            
            logger.log(f"[TextProcessing] Special split: {len(chunks)} video segments created. Distributing rest into {remaining_chunks_target} chunks.", level=LogLevel.DEBUG)
        else:
            remaining_chunks_target = num_chunks
            
        if not remaining_sentences:
            return chunks

        # --- PHASE 2: Even Distribution of Remaining Text ---
        # Calculate target length per chunk for remaining text
        total_len = sum(len(s) for s in remaining_sentences)
        target_chunk_len = total_len / remaining_chunks_target
        
        current_chunk = []
        current_len = 0
        
        chunks_added_in_phase2 = 0
        
        for sentence in remaining_sentences:
            sentence_len = len(sentence)
            
            # Decision: should we start a new chunk?
            if current_chunk and chunks_added_in_phase2 < remaining_chunks_target - 1:
                # Check if adding this sentence exceeds target by more than 30%
                if current_len + sentence_len > target_chunk_len * 1.3:
                    # Close current chunk and start new one
                    chunks.append(" ".join(current_chunk))
                    chunks_added_in_phase2 += 1
                    current_chunk = [sentence]
                    current_len = sentence_len
                    continue
            
            current_chunk.append(sentence)
            current_len += sentence_len
        
        # Add remaining sentences as the last chunk
        if current_chunk:
            chunks.append(" ".join(current_chunk))
        
        logger.log(f"[TextProcessing] Split text into {len(chunks)} chunks (requested: {num_chunks}, special: {special_count})", level=LogLevel.DEBUG)
        
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
        Aligns text segments to subtitle timings.
        
        Args:
            segments: List of text segments (from split_text_into_chunks).
            subtitles_path: Path to .ass or .srt subtitle file.
            task_id: For logging.
            
        Returns:
            List of dicts with timing info for each segment:
            [
                {"index": 0, "start": 0.0, "end": 5.2, "duration": 5.2, "confidence": 0.85},
                {"index": 1, "start": 5.2, "end": 12.1, "duration": 6.9, "confidence": 0.72},
                ...
            ]
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
        
        # Build a continuous text stream with time mappings
        text_stream, char_to_time = self._build_text_stream(subs)
        
        if not text_stream:
            logger.log(f"{prefix}[Sync] Empty text stream from subtitles", level=LogLevel.WARNING)
            return []
        
        # Normalize the stream for matching
        stream_normalized = self._normalize_text(text_stream)
        
        # Find each segment in the stream
        timings = []
        last_end_char = 0  # Track position to ensure segments are sequential
        last_end_time = 0.0  # Track time to ensure sequential timing
        
        for i, segment in enumerate(segments):
            seg_normalized = self._normalize_text(segment)
            
            if not seg_normalized:
                # Empty segment - give it minimal duration
                timings.append({
                    "index": i,
                    "start": last_end_time,
                    "end": last_end_time + 0.5,
                    "duration": 0.5,
                    "confidence": 0.0
                })
                last_end_time += 0.5
                continue
            
            # Find best match for this segment
            match_result = self._find_segment_in_stream(
                seg_normalized, 
                stream_normalized, 
                start_from=last_end_char
            )
            
            # FALLBACK: If no match found, try searching from earlier position
            # This handles cases where previous estimate pushed start_from too far
            if match_result is None and last_end_char > 0:
                # Try from 80% of last_end_char (go back a bit)
                fallback_start = max(0, int(last_end_char * 0.8))
                match_result = self._find_segment_in_stream(
                    seg_normalized,
                    stream_normalized,
                    start_from=fallback_start
                )
                
                # If still not found, try from even earlier
                if match_result is None:
                    fallback_start = max(0, int(last_end_char * 0.5))
                    match_result = self._find_segment_in_stream(
                        seg_normalized,
                        stream_normalized,
                        start_from=fallback_start
                    )
            
            # If match found but starts BEFORE last_end_time, try searching further
            if match_result:
                start_char, end_char, confidence = match_result
                start_time = self._char_to_time(start_char, char_to_time, 'start')
                
                # If this match overlaps with previous segment, search further
                if start_time < last_end_time - 0.5:  # Allow 0.5s tolerance
                    # Try searching from a later position
                    retry_start = last_end_char + len(seg_normalized) // 2
                    retry_result = self._find_segment_in_stream(
                        seg_normalized,
                        stream_normalized,
                        start_from=retry_start
                    )
                    if retry_result:
                        new_start_char, new_end_char, new_confidence = retry_result
                        new_start_time = self._char_to_time(new_start_char, char_to_time, 'start')
                        if new_start_time >= last_end_time - 0.5:
                            # Use the new match
                            start_char, end_char, confidence = new_start_char, new_end_char, new_confidence
                            start_time = new_start_time
            
            if match_result:
                start_char, end_char, confidence = match_result
                
                # Map character positions to times
                start_time = self._char_to_time(start_char, char_to_time, 'start')
                end_time = self._char_to_time(end_char, char_to_time, 'end')
                
                # Enforce sequential timing - start time cannot be before previous end
                if start_time < last_end_time:
                    start_time = last_end_time
                
                # Sanity check
                if end_time <= start_time:
                    end_time = start_time + 1.0
                
                timings.append({
                    "index": i,
                    "start": start_time,
                    "end": end_time,
                    "duration": end_time - start_time,
                    "confidence": confidence
                })
                
                last_end_char = end_char
                last_end_time = end_time
                
                logger.log(
                    f"{prefix}[Sync] Segment {i+1}/{len(segments)}: "
                    f"{start_time:.2f}s - {end_time:.2f}s (confidence: {confidence:.0%})",
                    level=LogLevel.DEBUG
                )
            else:
                # Fallback: estimate based on remaining time
                total_duration = subs[-1]['end'] if subs else 60.0
                remaining_segments = len(segments) - i
                remaining_time = total_duration - last_end_time
                
                estimated_duration = max(1.0, remaining_time / remaining_segments)
                
                timings.append({
                    "index": i,
                    "start": last_end_time,
                    "end": last_end_time + estimated_duration,
                    "duration": estimated_duration,
                    "confidence": 0.0
                })
                
                last_end_time += estimated_duration
                
                logger.log(
                    f"{prefix}[Sync] Segment {i+1}/{len(segments)}: "
                    f"No match found, using estimate: {last_end_time - estimated_duration:.2f}s - {last_end_time:.2f}s",
                    level=LogLevel.WARNING
                )
        
        # Final validation pass - ensure no overlaps and no gaps
        timings = self._fix_timing_overlaps(timings)
        
        logger.log(
            f"{prefix}[Sync] Aligned {len(timings)} segments. "
            f"Avg confidence: {sum(t['confidence'] for t in timings) / len(timings):.0%}",
            level=LogLevel.INFO
        )
        
        return timings
    
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
        Finds the best match for a segment in the text stream.
        Uses multiple anchor-based matching for better accuracy.
        
        This handles cases where:
        - Numbers are transcribed differently (70 vs Семьдесят)
        - Start of segment differs from whisper transcription
        
        Returns:
            (start_char, end_char, confidence) or None if no good match.
        """
        if not segment or not stream:
            return None
        
        # Limit search area to start_from onwards
        search_area = stream[start_from:]
        
        if not search_area:
            return None
        
        # Strategy 1: Try exact substring match first
        exact_pos = search_area.find(segment)
        if exact_pos != -1:
            abs_start = start_from + exact_pos
            abs_end = abs_start + len(segment)
            return (abs_start, abs_end, 1.0)
        
        # Strategy 2: Multi-anchor matching
        # Try to find matches from different parts of the segment
        # This helps when numbers are transcribed differently (70 vs Семьдесят)
        
        segment_words = segment.split()
        seg_len = len(segment)
        
        best_match = None
        best_confidence = 0.0
        
        # Special handling for SHORT segments (< 50 chars)
        # These need lower thresholds and simpler matching
        is_short_segment = seg_len < 50
        base_threshold = 0.4 if is_short_segment else 0.5
        
        # Define multiple anchor positions to try
        anchors_to_try = []
        
        # Anchor 1: Start of segment (first 20-40 chars)
        anchor_len = min(seg_len, 40)
        anchors_to_try.append(('start', segment[:anchor_len], 0))
        
        # Anchor 2: Middle of segment
        if seg_len > 60:
            mid_start = seg_len // 3
            mid_anchor = segment[mid_start:mid_start + 40]
            anchors_to_try.append(('middle', mid_anchor, mid_start))
        
        # Anchor 3: End of segment (last 20-40 chars) 
        if seg_len > 40:
            end_anchor = segment[-40:]
            anchors_to_try.append(('end', end_anchor, seg_len - 40))
        
        # Anchor 4: Try to find distinctive phrases (words 3-6)
        if len(segment_words) > 5:
            phrase_start = len(' '.join(segment_words[:2])) + 1
            phrase = ' '.join(segment_words[2:6])
            if len(phrase) >= 15:
                anchors_to_try.append(('phrase', phrase, phrase_start))
        
        # For short segments, also try first 2-3 words as anchor
        if is_short_segment and len(segment_words) >= 2:
            first_words = ' '.join(segment_words[:min(3, len(segment_words))])
            if len(first_words) >= 8:
                anchors_to_try.append(('first_words', first_words, 0))
        
        for anchor_name, anchor_text, anchor_offset in anchors_to_try:
            # Try to find this anchor in the search area
            match = self._fuzzy_find(anchor_text, search_area, threshold=base_threshold)
            
            if match is None:
                # Try with shorter version and lower threshold
                short_anchor = anchor_text[:min(len(anchor_text), 25)]
                match = self._fuzzy_find(short_anchor, search_area, threshold=base_threshold - 0.1)
            
            if match:
                match_pos, confidence = match
                
                # Calculate estimated segment boundaries based on where this anchor was found
                if anchor_name == 'start':
                    est_start = match_pos
                    est_end = match_pos + int(seg_len * 1.1)
                elif anchor_name == 'end':
                    est_end = match_pos + len(anchor_text)
                    est_start = max(0, match_pos - (seg_len - len(anchor_text)))
                else:  # middle or phrase
                    est_start = max(0, match_pos - anchor_offset)
                    est_end = est_start + int(seg_len * 1.1)
                
                # Validate: est_end should be after est_start
                if est_end <= est_start:
                    est_end = est_start + int(seg_len * 1.1)
                
                # Keep track of best match
                if confidence > best_confidence:
                    best_confidence = confidence
                    abs_start = start_from + est_start
                    abs_end = start_from + min(est_end, len(search_area))
                    best_match = (abs_start, abs_end, confidence)
        
        if best_match:
            return best_match
        
        # Strategy 3: Last resort - try very short distinctive words
        # Find any word longer than 6 characters that might be unique
        for word in segment_words:
            if len(word) >= 7:  # Only try distinctive longer words
                word_lower = word.lower()
                pos = search_area.lower().find(word_lower)
                if pos != -1:
                    # Found a distinctive word, estimate segment position
                    word_offset = segment.lower().find(word_lower)
                    est_start = max(0, pos - word_offset)
                    est_end = est_start + int(seg_len * 1.1)
                    
                    abs_start = start_from + est_start
                    abs_end = start_from + min(est_end, len(search_area))
                    
                    return (abs_start, abs_end, 0.5)  # Lower confidence for word-only match
        
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
    
    def _similarity(self, s1: str, s2: str) -> float:
        """
        Computes similarity ratio between two strings.
        Uses a simple character-matching approach for speed.
        """
        if not s1 or not s2:
            return 0.0
        
        if s1 == s2:
            return 1.0
        
        # Simple matching characters count
        matches = sum(c1 == c2 for c1, c2 in zip(s1, s2))
        return matches / max(len(s1), len(s2))
    
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
                # Close the gap by extending the previous segment
                # (alternatively could split the gap, but extending is more natural)
                prev['end'] = current['start']
                prev['duration'] = prev['end'] - prev['start']
            
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
        
        # Remove all punctuation and symbols (Unicode-aware)
        # \p{P} = punctuation, \p{S} = symbols
        # Since we can't use regex module, use a character class approach
        # Remove common punctuation
        punctuation = (
            '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'  # ASCII punctuation
            '。！？、，；：""''【】（）…—·'  # Chinese/Japanese punctuation
            '؟،؛'  # Arabic punctuation
            '।॥'  # Devanagari punctuation
            '「」『』〈〉《》〔〕'  # More CJK brackets
        )
        
        for char in punctuation:
            text = text.replace(char, ' ')
        
        # Lowercase
        text = text.lower()
        
        # Collapse whitespace
        text = ' '.join(text.split())
        
        return text.strip()
    
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
                        r'(\d{2}):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})[,.](\d{3})',
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
