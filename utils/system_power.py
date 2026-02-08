
import os
import platform
import subprocess
from utils.logger import logger, LogLevel

def shutdown_system():
    """Shuts down the system."""
    system = platform.system()
    try:
        if system == "Windows":
            # /s = shutdown, /f = force close apps, /t 3 = wait 3 seconds (UI handled main delay)
            subprocess.run(["shutdown", "/s", "/f", "/t", "3"], check=True)
        elif system == "Darwin":
            # macOS shutdown
            subprocess.run(["osascript", "-e", 'tell app "System Events" to shut down'], check=True)
        elif system == "Linux":
             subprocess.run(["shutdown", "now"], check=True)
        else:
            logger.log(f"Shutdown not supported on {system}", level=LogLevel.WARNING)
    except Exception as e:
         logger.log(f"Error initiating shutdown: {e}", level=LogLevel.ERROR)

def hibernate_system():
    """Hibernates the system."""
    system = platform.system()
    try:
        if system == "Windows":
            subprocess.run(["shutdown", "/h"], check=True)
        elif system == "Darwin":
             # macOS safe sleep (closest to hibernate)
             # pmset -a hibernatemode 25
             # But usually just sleep is enough. True hibernate is harder to trigger via command without sudo.
             # We will fallback to sleep for macOS if hibernate is requested, or try 'pmset'
             logger.log("Hibernate on macOS is complex without root. Falling back to sleep.", level=LogLevel.WARNING)
             sleep_system()
        else:
             logger.log(f"Hibernate not supported on {system}", level=LogLevel.WARNING)
    except Exception as e:
         logger.log(f"Error initiating hibernate: {e}", level=LogLevel.ERROR)

def sleep_system():
    """Puts the system to sleep."""
    system = platform.system()
    try:
        if system == "Windows":
            # rundll32.exe powrprof.dll,SetSuspendState 0,1,0
            # Note: This might hibernate if hibernation is enabled.
            # Using specific powershell command is better if available, but rundll32 is standard.
            subprocess.run(["rundll32.exe", "powrprof.dll,SetSuspendState", "0,1,0"], check=True)
        elif system == "Darwin":
            subprocess.run(["pmset", "sleepnow"], check=True)
        else:
            logger.log(f"Sleep not supported on {system}", level=LogLevel.WARNING)
    except Exception as e:
         logger.log(f"Error initiating sleep: {e}", level=LogLevel.ERROR)

def perform_power_action(action_name):
    """Performs the specified power action: 'shutdown', 'hibernate', 'sleep'."""
    logger.log(f"Initiating system power action: {action_name}", level=LogLevel.INFO)
    
    if action_name == 'shutdown':
        shutdown_system()
    elif action_name == 'hibernate':
        hibernate_system()
    elif action_name == 'sleep':
        sleep_system()
    else:
        logger.log(f"Unknown power action: {action_name}", level=LogLevel.WARNING)
