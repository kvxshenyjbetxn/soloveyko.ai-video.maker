# add_test_data.py
import os
import random
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

    # Simulate data for the last 60 days
    now = datetime.now()
    start_date = now - timedelta(days=60)
    
    for i in range(60):
        current_date = start_date + timedelta(days=i)
        if current_date > now:
            break
            
        for event_type in event_types:
            # Add a random number of events for each type, up to 10
            for _ in range(random.randint(0, 10)):
                statistics_manager.record_event(event_type, timestamp=current_date)

    print("Sample data added successfully.")
    print("You can now restart the main application and check the Statistics tab.")

if __name__ == "__main__":
    # The goal is to ensure a clean slate and then add fresh sample data.
    # Instead of deleting the file, which can fail due to locks,
    # we will clear the data from the tables directly.
    try:
        print("Initializing StatisticsManager and clearing all existing data...")
        # This will create the DB and tables if they don't exist, or connect if they do.
        statistics_manager.init_db() 
        # Now, clear all data from the tables.
        statistics_manager.clear_all_data()
        
        print("Adding new sample data...")
        add_sample_data()
        
        print("Script completed successfully.")
    except Exception as e:
        print(f"An error occurred during the data refresh process: {e}")
        # Optionally, re-raise the exception if you want the script to exit with an error code
        # raise