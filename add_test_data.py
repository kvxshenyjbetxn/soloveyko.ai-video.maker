# add_test_data.py
import os
from datetime import datetime, timedelta
import time # Import time for sleep
# Assuming statistics_manager is initialized at the module level in core/statistics_manager.py
# and can be imported directly.
from core.statistics_manager import statistics_manager

def add_sample_data():
    print("Adding sample statistics data...")

    event_types = [
        'translation',
        'image_prompts',
        'image_generation_stage',
        'image_generated',
        'voiceover',
        'subtitles',
        'montage',
        'custom_stage_review' # Example custom stage
    ]

    # Simulate data for the last 5 days
    for i in range(5, 0, -1): # From 5 days ago up to yesterday
        current_date = datetime.now() - timedelta(days=i)
        
        for _ in range(5): # Record 5 events per day
            for event_type in event_types:
                statistics_manager.record_event(event_type, timestamp=current_date)

    # Add some data for today as well
    for _ in range(3):
        for event_type in event_types:
            statistics_manager.record_event(event_type, timestamp=datetime.now())

    print("Sample data added successfully.")
    print("You can now restart the main application and check the Statistics tab.")

if __name__ == "__main__":
    db_path = 'assets/statistics.db'
    
    # Try to delete existing DB
    if os.path.exists(db_path):
        print(f"Attempting to delete existing database '{db_path}'...")
        try:
            os.remove(db_path)
            print("Existing database deleted.")
        except PermissionError:
            print(f"ERROR: Could not delete '{db_path}'. It might be in use. Trying to proceed without deletion.")
            # If we can't delete, we might hit unique constraint errors later
            # This is a fallback if the user couldn't manually delete it.

    # Retry mechanism for DB creation and data addition
    max_retries = 5
    retry_delay = 1 # seconds
    for attempt in range(max_retries):
        try:
            print(f"Attempt {attempt + 1}/{max_retries} to initialize DB and add data...")
            # Ensure a fresh StatisticsManager instance
            # This will create the DB file if it doesn't exist or connect to existing
            statistics_manager.__init__() 
            add_sample_data()
            print("Script completed successfully.")
            break # Exit retry loop on success
        except Exception as e: # Catch broad exception to include PermissionError, etc.
            print(f"Error during DB operation: {e}")
            if attempt < max_retries - 1:
                print(f"Retrying in {retry_delay} seconds...")
                time.sleep(retry_delay)
            else:
                print("Max retries reached. Script failed.")
                raise # Re-raise the last exception if all retries fail