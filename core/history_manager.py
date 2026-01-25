import json
import os
import sys
import threading
import platform
from datetime import datetime, timedelta
from utils.logger import logger, LogLevel

class HistoryManager:
    def __init__(self, history_dir='history'):
        self.lock = threading.Lock()
        
        if platform.system() == "Darwin":
            base_dir = os.path.expanduser("~/Library/Application Support/Soloveyko.AI-Video.Maker")
        elif getattr(sys, 'frozen', False):
            # Running as a bundled exe (Windows)
            base_dir = os.path.dirname(sys.executable)
        else:
            # Running as a script
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

        self.history_path = os.path.join(base_dir, history_dir)
        os.makedirs(self.history_path, exist_ok=True)
        self._cleanup()

    def add_entry(self, state):
        with self.lock:
            try:
                today_str = datetime.now().strftime('%Y-%m-%d')
                file_path = os.path.join(self.history_path, f"history_{today_str}.json")
                
                # Language-specific data
                lang_entry = {
                    "lang_id": state.lang_id,
                    "lang_name": state.lang_name,
                    "stages": state.stages,
                    "status": state.status,
                    "template": state.lang_data.get('template_name', 'Default'),
                    "start_time": state.start_time.isoformat() if state.start_time else None,
                    "end_time": datetime.now().isoformat(),
                    "original_text": state.original_text_preview or state.original_text,
                    "translated_text": state.translated_text_preview
                }

                data = []
                if os.path.exists(file_path):
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            data = json.load(f)
                    except (json.JSONDecodeError, FileNotFoundError):
                        data = []
                
                # Grouping logic: find if there's an existing entry for this job within the same "session" (e.g., started around the same time)
                # Or simply by job_name if it's recent enough (e.g., within 1 hour)
                existing_entry = None
                if state.start_time:
                    for entry in reversed(data):
                        if entry.get('job_name') == state.job_name:
                            # Check if the existing entry's start_time is close to current task's start_time
                            try:
                                entry_start = datetime.fromisoformat(entry.get('start_time', ''))
                                if abs((state.start_time - entry_start).total_seconds()) < 3600: # 1 hour window
                                    existing_entry = entry
                                    break
                            except:
                                continue

                if existing_entry:
                    # Append language if not already there
                    if 'languages' not in existing_entry:
                        existing_entry['languages'] = []
                    
                    # Update or add lang_entry
                    found_lang = False
                    for i, l in enumerate(existing_entry['languages']):
                        if l.get('lang_id') == state.lang_id:
                            existing_entry['languages'][i] = lang_entry
                            found_lang = True
                            break
                    if not found_lang:
                        existing_entry['languages'].append(lang_entry)
                        
                    # Update job end_time to the latest
                    existing_entry['end_time'] = lang_entry['end_time']
                else:
                    # Create new job entry
                    new_job_entry = {
                        "job_name": state.job_name,
                        "start_time": state.start_time.isoformat() if state.start_time else lang_entry['start_time'],
                        "end_time": lang_entry['end_time'],
                        "languages": [lang_entry]
                    }
                    data.append(new_job_entry)
                
                with open(file_path, 'w', encoding='utf-8') as f:
                    json.dump(data, f, indent=4, ensure_ascii=False)
                
                logger.log(f"History entry added/updated for job {state.job_name} ({state.lang_id})", level=LogLevel.INFO)
            except Exception as e:
                logger.log(f"Failed to add history entry: {e}", level=LogLevel.ERROR)

    def get_history(self, days=30):
        with self.lock:
            history = []
            end_date = datetime.now()
            start_date = end_date - timedelta(days=days)
            
            # Ensure we check all relevant files in the directory
            if not os.path.exists(self.history_path):
                return []

            for filename in os.listdir(self.history_path):
                if filename.startswith("history_") and filename.endswith(".json"):
                    try:
                        date_str = filename[8:-5]
                        file_date = datetime.strptime(date_str, '%Y-%m-%d')
                        if file_date >= start_date.replace(hour=0, minute=0, second=0, microsecond=0):
                             file_path = os.path.join(self.history_path, filename)
                             try:
                                 with open(file_path, 'r', encoding='utf-8') as f:
                                     history.extend(json.load(f))
                             except Exception as e:
                                 logger.log(f"Failed to read history file {file_path}: {e}", level=LogLevel.ERROR)
                    except ValueError:
                        continue
            
            # Sort by end_time descending
            history.sort(key=lambda x: x.get('end_time', ''), reverse=True)
            return history

    def clear_history(self):
        with self.lock:
            try:
                for filename in os.listdir(self.history_path):
                    if filename.startswith("history_") and filename.endswith(".json"):
                        os.remove(os.path.join(self.history_path, filename))
                logger.log("History cleared.", level=LogLevel.INFO)
            except Exception as e:
                logger.log(f"Error clearing history: {e}", level=LogLevel.ERROR)

    def _cleanup(self):
        try:
            now = datetime.now()
            retention_limit = now - timedelta(days=30)
            
            if not os.path.exists(self.history_path):
                return

            for filename in os.listdir(self.history_path):
                if filename.startswith("history_") and filename.endswith(".json"):
                    try:
                        date_str = filename[8:-5]
                        file_date = datetime.strptime(date_str, '%Y-%m-%d')
                        if file_date < retention_limit.replace(hour=0, minute=0, second=0, microsecond=0):
                            os.remove(os.path.join(self.history_path, filename))
                            logger.log(f"Deleted old history file: {filename}", level=LogLevel.INFO)
                    except ValueError:
                        continue
        except Exception as e:
            logger.log(f"Error during history cleanup: {e}", level=LogLevel.ERROR)

    def register_recent_job(self, job):
        """Saves a job to the recent jobs list for recovery."""
        with self.lock:
            try:
                file_path = os.path.join(self.history_path, "recent_jobs.json")
                recent_jobs = []
                if os.path.exists(file_path):
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            recent_jobs = json.load(f)
                    except:
                        recent_jobs = []
                
                # Add timestamp if not present
                if 'created_at' not in job:
                    job['created_at'] = datetime.now().isoformat()
                
                # Prevent duplicates by name and created_at (or just name if very recent)
                # For simplicity, just append and then clean up old ones
                recent_jobs.append(job)
                
                # Keep only last 2 days or last 100 jobs
                now = datetime.now()
                two_days_ago = now - timedelta(days=2)
                
                filtered_jobs = []
                for j in recent_jobs:
                    try:
                        created_at = datetime.fromisoformat(j.get('created_at', ''))
                        if created_at >= two_days_ago:
                            filtered_jobs.append(j)
                    except:
                        continue
                
                # Limit count
                if len(filtered_jobs) > 100:
                    filtered_jobs = filtered_jobs[-100:]
                
                with open(file_path, 'w', encoding='utf-8') as f:
                    json.dump(filtered_jobs, f, indent=4, ensure_ascii=False)
                
            except Exception as e:
                logger.log(f"Failed to register recent job: {e}", level=LogLevel.ERROR)

    def get_recent_jobs(self, days=2):
        """Returns recent jobs for recovery."""
        with self.lock:
            file_path = os.path.join(self.history_path, "recent_jobs.json")
            if not os.path.exists(file_path):
                return []
            
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    jobs = json.load(f)
                
                now = datetime.now()
                limit = now - timedelta(days=days)
                
                filtered = []
                for j in reversed(jobs): # Newest first
                    try:
                        created_at = datetime.fromisoformat(j.get('created_at', ''))
                        if created_at >= limit:
                            filtered.append(j)
                    except:
                        continue
                return filtered
            except Exception as e:
                logger.log(f"Failed to get recent jobs: {e}", level=LogLevel.ERROR)
                return []

history_manager = HistoryManager()
