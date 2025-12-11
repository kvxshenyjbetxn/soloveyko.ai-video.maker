import sqlite3
import os
from datetime import datetime
import shutil

class StatisticsManager:
    def __init__(self, db_path='assets/statistics.db'):
        self.db_path = db_path
        
        # Ensure the assets directory exists
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        # Move old database if it exists
        old_db_path = 'statistics.db'
        if os.path.exists(old_db_path) and not os.path.exists(self.db_path):
            try:
                shutil.move(old_db_path, self.db_path)
                print(f"Moved database from '{old_db_path}' to '{self.db_path}'")
            except Exception as e:
                print(f"Error moving database: {e}")

        self._check_and_create_db()
        self._migrate_db()

    def _check_and_create_db(self):
        if not os.path.exists(self.db_path):
            self.init_db()

    def _migrate_db(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='events'")
            events_table = cursor.fetchone()
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='event_types'")
            event_types_table = cursor.fetchone()

            if events_table and not event_types_table:
                cursor.execute("PRAGMA table_info(events)")
                columns = [row[1] for row in cursor.fetchall()]
                if 'event_type_id' not in columns:
                    print("Migrating database schema...")
                    try:
                        cursor.execute("ALTER TABLE events RENAME TO events_old")
                        
                        self._create_tables(cursor)

                        cursor.execute("SELECT DISTINCT event_type FROM events_old")
                        old_event_types = cursor.fetchall()
                        event_type_map = {}
                        for event_type_row in old_event_types:
                            event_type = event_type_row[0]
                            cursor.execute("INSERT INTO event_types (name) VALUES (?)", (event_type,))
                            event_type_map[event_type] = cursor.lastrowid

                        cursor.execute("SELECT event_type, timestamp FROM events_old")
                        old_events = cursor.fetchall()
                        for old_event in old_events:
                            event_type, timestamp = old_event
                            event_type_id = event_type_map[event_type]
                            cursor.execute("INSERT INTO events (event_type_id, timestamp) VALUES (?, ?)", (event_type_id, timestamp))

                        cursor.execute("DROP TABLE events_old")
                        conn.commit()
                        print("Database migration completed.")
                    except Exception as e:
                        print(f"Error migrating database: {e}")
                        conn.rollback()


    def init_db(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            self._create_tables(cursor)
            conn.commit()

    def _create_tables(self, cursor):
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS event_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )
        ''')
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type_id INTEGER NOT NULL,
                timestamp DATETIME NOT NULL,
                FOREIGN KEY (event_type_id) REFERENCES event_types (id)
            )
        ''')

    def record_event(self, event_type, timestamp=None):
        if timestamp is None:
            timestamp = datetime.now()
        # The 'timeout' parameter is important for multi-threaded access.
        # It will make the connection wait for the specified amount of time if the database is locked.
        with sqlite3.connect(self.db_path, timeout=10) as conn:
            cursor = conn.cursor()
            
            # Use "INSERT OR IGNORE" to atomically create the event type if it doesn't exist.
            # This is crucial to prevent race conditions in a multi-threaded environment.
            cursor.execute("INSERT OR IGNORE INTO event_types (name) VALUES (?)", (event_type,))
            
            # Now, we are guaranteed that the event type exists. Fetch its ID.
            cursor.execute("SELECT id FROM event_types WHERE name = ?", (event_type,))
            event_type_id = cursor.fetchone()[0]
            
            # Record the actual event.
            cursor.execute("INSERT INTO events (event_type_id, timestamp) VALUES (?, ?)", (event_type_id, timestamp))
            # The 'with' statement automatically handles committing the transaction.

    def get_statistics(self, period='all_time'):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            query = """
                SELECT et.name, COUNT(e.id) 
                FROM events e
                JOIN event_types et ON e.event_type_id = et.id
            """

            now = datetime.now()
            if period == 'current_month':
                start_date = now.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
                query += f" WHERE e.timestamp >= '{start_date}'"
            elif period == 'current_year':
                start_date = now.replace(month=1, day=1, hour=0, minute=0, second=0, microsecond=0)
                query += f" WHERE e.timestamp >= '{start_date}'"
            
            query += " GROUP BY et.name"
            
            cursor.execute(query)
            return dict(cursor.fetchall())

    def get_daily_statistics_by_event(self, period='all_time'):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            query = """
                SELECT et.name, date(e.timestamp), COUNT(e.id)
                FROM events e
                JOIN event_types et ON e.event_type_id = et.id
            """

            now = datetime.now()
            if period == 'current_month':
                start_date = now.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
                query += f" WHERE e.timestamp >= '{start_date}'"
            elif period == 'current_year':
                start_date = now.replace(month=1, day=1, hour=0, minute=0, second=0, microsecond=0)
                query += f" WHERE e.timestamp >= '{start_date}'"
            
            query += " GROUP BY et.name, date(e.timestamp) ORDER BY date(e.timestamp)"
            
            cursor.execute(query)
            
            stats_by_event = {}
            for event_name, date_str, count in cursor.fetchall():
                if event_name not in stats_by_event:
                    stats_by_event[event_name] = []
                stats_by_event[event_name].append((date_str, count))
            
            return stats_by_event


statistics_manager = StatisticsManager()