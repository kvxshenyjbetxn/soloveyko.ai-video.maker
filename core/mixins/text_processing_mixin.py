import re
import difflib
from typing import List, Dict, Any
from utils.logger import logger, LogLevel

class TextProcessingMixin:
    """
    Mixin for TaskProcessor to handle text segmentation and subtitle alignment.
    """
    
    def split_text_into_chunks(self, text: str, num_chunks: int) -> List[str]:
        """
        Splits text into `num_chunks` roughly equal parts, respecting sentence boundaries.
        """
        if not text:
            return []
        
        if num_chunks <= 1:
            return [text]

        # 1. Split into sentences (simple regex for ., !, ?)
        # We want to keep the punctuation with the sentence.
        # (?<=[.!?]) looks behind for punctuation.
        sentences = re.split(r'(?<=[.!?])\s+', text)
        sentences = [s.strip() for s in sentences if s.strip()]
        
        if not sentences:
            return [text]
            
        total_len = sum(len(s) for s in sentences)
        target_chunk_len = total_len / num_chunks
        
        chunks = []
        current_chunk = []
        current_len = 0
        
        for sentence in sentences:
            # If adding this sentence makes us exceed target significantly, close chunk
            # But ensure at least one sentence if we are starting a new chunk
            if current_chunk and (current_len + len(sentence)) > target_chunk_len and len(chunks) < num_chunks - 1:
                chunks.append(" ".join(current_chunk))
                current_chunk = [sentence]
                current_len = len(sentence)
            else:
                current_chunk.append(sentence)
                current_len += len(sentence)
        
        if current_chunk:
            chunks.append(" ".join(current_chunk))
            
        return chunks

    def align_images_to_subtitles(self, segments_map: Dict[int, str], subtitles_path: str) -> List[float]:
        """
        Calculates durations for each image segment based on subtitles.
        
        Args:
            segments_map: Dictionary mapping image index (0-based) to the text segment it represents.
            subtitles_path: Path to the .ass or .srt file.
            
        Returns:
            List of durations (in seconds) for each image.
        """
        import os
        from core.subtitle_engine import SubtitleEngine
        
        if not os.path.exists(subtitles_path):
            return []

        # We can reuse SubtitleEngine's parsing logic
        # Since this method is static-like or utility, we instantiate a temp engine or make parser static
        # But SubtitleEngine methods currently need 'self'. Let's instantiate lightweight.
        engine = SubtitleEngine()
        
        # Determine format
        if subtitles_path.endswith('.ass'):
             # ASS parsing is complex, maybe rely on reading raw or if we have a parser
             # The provided SubtitleEngine has _parse_srt but not public _parse_ass (it generates it).
             # However, usually we generate .ass from .srt or internal segments. 
             # If we have the original segments in memory that would be best, but we might only have the file now.
             # Let's try to parse ASS roughly or hope we have SRT available (usually we generate both or temp srt).
             # CHECK: TaskProcessor generates .ass. Does it keep .srt? 
             # The SubtitleWorker generates .ass. Logic in SubtitleEngine._get_segments gets segments first.
             # Maybe we can modify SubtitleWorker to save segments to TaskState? That would be much more reliable.
             # For now, let's assume we might need to parse the text file if state is lost, but 
             # IMPROVEMENT: Let's assume we can parse ASS or SRT. 
             # Actually, let's implement a simple ASS reader here since we only need events.
             subs = self._parse_ass_events(subtitles_path)
        else:
            subs = engine._parse_srt(subtitles_path)
            
        if not subs:
            logger.log(f"Subtitles parsed but list is empty for path: {subtitles_path}", level=LogLevel.WARNING)
            return []

        # Flatten subs text for searching
        # We need to find where each segment starts and ends in the subtitle stream.
        # Strategy: Normalized text matching.
        
        durations = []
        
        total_subs = len(subs)
        current_sub_idx = 0
        
        sorted_indices = sorted(segments_map.keys())
        
        for i in sorted_indices:
            segment_text = segments_map[i]
            # Normalize
            seg_norm = self._normalize_text(segment_text)
            
            start_time = -1.0
            end_time = -1.0
            
            # Find start
            # We look for the first subtitle that contains the approximate start of our segment
            # OR we just accumulate subtitles until we "cover" the segment.
            
            matched_subs_count = 0
            accumulated_text = ""
            
            # Start search from where previous left off
            search_idx = current_sub_idx
            
            best_start_idx = search_idx
            
            # Heuristic: The segment should match a sequence of subtitles.
            # We iterate subtitles and try to fuzzy match the segment.
            # But simple approach: 
            # 1. Find the first subtitle that has a significant overlap with the start of the segment.
            
            # Let's try a word-bag approach or sliding window?
            # Simpler: The segments preserve order. 
            # So current segment MUST start at or after current_sub_idx.
            
            if search_idx >= total_subs:
                durations.append(5.0) # Fallback
                continue

            start_time = subs[search_idx]['start']
            
            # Find end
            # We greedily consume subtitles until we have "enough" text to match the segment
            # or until we reach the start of the next segment (if we had it).
            # But we only have current segment.
            
            # Better approach:
            # Calculate match score for every subtitle against current segment.
            # This is hard because one segment = many subtitles.
            
            # Approach 2: Text cleanup mapping.
            # Construct full text from subtitles with mapping to timestamps.
            # Full Text: "Hello world. This is a test." -> [(0.0, 1.5, "Hello world."), (1.5, 3.0, "This is a test.")]
            # Segment: "Hello world. This is a test."
            # We find the character range of the segment in the full text.
            
            full_sub_text = ""
            sub_char_map = [] # list of (char_index, timestamp_start, timestamp_end)
            
            # We only build this once? No, maybe per segment search to be simpler but less efficient.
            # Let's build full string from current_sub_idx onwards.
            
            temp_text_buffer = ""
            temp_map = []
            
            for k in range(current_sub_idx, total_subs):
                s_text = self._normalize_text(subs[k]['text']) + " "
                s_start = subs[k]['start']
                s_end = subs[k]['end']
                
                start_char = len(temp_text_buffer)
                temp_text_buffer += s_text
                end_char = len(temp_text_buffer)
                
                # Map this range to the subtitle time
                temp_map.append({
                    'start_char': start_char,
                    'end_char': end_char,
                    'time_start': s_start,
                    'time_end': s_end
                })
            
            # Now find segment_text in temp_text_buffer
            # Fuzzy approximate? 
            # If we split by sentences, it should be fairly exact if normalization is good.
            # Let's assume exact match first, then fallback.
            
            # Strategy: Anchor-based matching.
            # Instead of exact matching the whole block (which fails if one word is wrong),
            # we look for the START of the segment and the END of the segment.
            
            # 1. Try to find the start of the segment
            start_anchor_len = min(len(seg_norm), 30)
            start_anchor = seg_norm[:start_anchor_len]
            
            # Find start anchor in buffer
            matcher_start = difflib.SequenceMatcher(None, temp_text_buffer, start_anchor)
            match_start = matcher_start.find_longest_match(0, len(temp_text_buffer), 0, len(start_anchor))
            
            # If start anchor is not found with good confidence, try fuzzy
            match_start_idx = -1
            if match_start.size > len(start_anchor) * 0.6:
                match_start_idx = match_start.a
            else:
                 # If finding start fails, maybe we just continue from current?
                 match_start_idx = 0
            
            # 2. Try to find the end of the segment
            # We search AFTER the start
            search_end_buffer = temp_text_buffer[match_start_idx:]
            end_anchor_len = min(len(seg_norm), 30)
            end_anchor = seg_norm[-end_anchor_len:]
            
            matcher_end = difflib.SequenceMatcher(None, search_end_buffer, end_anchor)
            match_end = matcher_end.find_longest_match(0, len(search_end_buffer), 0, len(end_anchor))
             
            match_end_idx = -1
            if match_end.size > len(end_anchor) * 0.6:
                 # match_end.a is relative to search_end_buffer
                 # We want the END of the match
                 match_end_idx = match_start_idx + match_end.a + match_end.size
            else:
                # If end not found, maybe the segment goes until the buffer ends (if it's the last one)
                # or we estimate based on length ratio?
                # Let's try to assume it matches a proportional amount of text
                # But safer to just look for the NEXT segment's start?
                # For now, let's use a loose fallback: find *something*
                pass
            
            # Refined Start/End Logic
            if match_start_idx != -1 and match_end_idx != -1:
                 # We have character ranges in temp_text_buffer
                 # Map chars back to timestamps
                
                 t_start = -1
                 t_end = -1
                 last_used_rel_idx = 0
                 
                 for idx, item in enumerate(temp_map):
                     # item uses absolute chars relative to temp_text_buffer start
                     if item['end_char'] > match_start_idx and t_start == -1:
                         t_start = item['time_start']
                     
                     if item['start_char'] < match_end_idx:
                         t_end = item['time_end']
                         last_used_rel_idx = idx

                 start_time = t_start
                 end_time = t_end
                 
                 # Advance current_sub_idx
                 # We consumed up to last_used_rel_idx
                 current_sub_idx += last_used_rel_idx + 1
                 
            else:
                # Fallback to simple contiguous matching if anchors fail (e.g. short texts)
                # Or relying on previous logic flow
                matcher = difflib.SequenceMatcher(None, temp_text_buffer, seg_norm)
                match = matcher.find_longest_match(0, len(temp_text_buffer), 0, len(seg_norm))
                
                if match.size > len(seg_norm) * 0.4:
                     match_start_idx = match.a
                     match_end_idx = match.a + match.size
                     # Duplicate mapping logic (can be refactored)
                     t_start = -1
                     t_end = -1
                     last_used_rel_idx = 0
                     for idx, item in enumerate(temp_map):
                         if item['end_char'] > match_start_idx and t_start == -1:
                             t_start = item['time_start']
                         if item['start_char'] < match_end_idx:
                             t_end = item['time_end']
                             last_used_rel_idx = idx
                     start_time = t_start
                     end_time = t_end
                     current_sub_idx += last_used_rel_idx + 1
                else:
                    # COMPLETE FALLBACK
                    # Just take the next unassigned subtitle?
                    if current_sub_idx < total_subs:
                        start_time = subs[current_sub_idx]['start']
                        end_time = subs[current_sub_idx]['end']
                        current_sub_idx += 1
            
            if start_time != -1 and end_time != -1:
                durations.append(max(0.1, end_time - start_time))
            else:
                durations.append(5.0)

        return durations

    def _normalize_text(self, text):
        return re.sub(r'[^\w\s]', '', text.lower()).replace('\n', ' ').strip()

    def _parse_ass_events(self, path):
        # Very basic ASS event parser
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
                
                if in_events:
                    if line.startswith('Format:'):
                        format_spec = [x.strip() for x in line[7:].split(',')]
                    elif line.startswith('Dialogue:'):
                        parts = line[9:].split(',', len(format_spec)-1)
                        if len(parts) == len(format_spec):
                            row = dict(zip(format_spec, parts))
                            events.append({
                                'start': self._ass_time_to_sec(row.get('Start', '0:00:00.00')),
                                'end': self._ass_time_to_sec(row.get('End', '0:00:00.00')),
                                'text': row.get('Text', '')
                            })
        except Exception:
            return []
        return events

    def _ass_time_to_sec(self, t_str):
        try:
            h, m, s = t_str.split(':')
            return int(h) * 3600 + int(m) * 60 + float(s)
        except:
            return 0.0
