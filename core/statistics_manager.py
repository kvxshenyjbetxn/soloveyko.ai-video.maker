import json
import os
from datetime import datetime, timedelta
import threading
import sys

class StatisticsManager:
    def __init__(self, db_name='statistics.json'):
        self.lock = threading.Lock()
        
        if getattr(sys, 'frozen', False):
            # Running as a bundled exe
            base_dir = os.path.dirname(sys.executable)
        else:
            # Running as a script
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

        self.json_path = os.path.join(base_dir, db_name)
        
        self.__ensure_file_exists_nolock()

    def __ensure_file_exists_nolock(self):
        if not os.path.exists(self.json_path):
            with open(self.json_path, 'w') as f:
                json.dump({"daily_video_counts": {}}, f)

    def record_video_creation(self):
        with self.lock:
            today_str = datetime.now().strftime('%d-%m-%Y')
            
            data = {}
            if os.path.exists(self.json_path):
                try:
                    with open(self.json_path, 'r') as f:
                        data = json.load(f)
                except (json.JSONDecodeError, FileNotFoundError):
                    # File is empty or corrupted, start fresh
                    data = {"daily_video_counts": {}}
            else:
                 data = {"daily_video_counts": {}}

            if "daily_video_counts" not in data:
                 data["daily_video_counts"] = {}

            counts = data.get("daily_video_counts", {})
            counts[today_str] = counts.get(today_str, 0) + 1
            data["daily_video_counts"] = counts
            
            with open(self.json_path, 'w') as f:
                json.dump(data, f, indent=4)

    def get_daily_video_counts(self):
        with self.lock:
            if not os.path.exists(self.json_path):
                return {}

            try:
                with open(self.json_path, 'r') as f:
                    data = json.load(f)
            except (json.JSONDecodeError, FileNotFoundError):
                return {}

            counts = data.get("daily_video_counts", {})
            if not counts:
                return {}

            # Find the date range
            date_keys = [datetime.strptime(d, '%d-%m-%Y') for d in counts.keys()]
            start_date = min(date_keys)
            end_date = max(date_keys)
            
            # Fill in missing dates
            filled_counts = {}
            current_date = start_date
            while current_date <= end_date:
                date_str = current_date.strftime('%d-%m-%Y')
                filled_counts[date_str] = counts.get(date_str, 0)
                current_date += timedelta(days=1)
            
            return filled_counts

    def clear_all_data(self):
        with self.lock:
            if os.path.exists(self.json_path):
                try:
                    os.remove(self.json_path)
                    print("Statistics file has been cleared.")
                except OSError as e:
                    print(f"Error clearing statistics file: {e}")
            self.__ensure_file_exists_nolock() # Recreate the empty file

statistics_manager = StatisticsManager()