import requests
from utils.logger import logger, LogLevel
from utils.settings import settings_manager

# CONSTANTS - You should probably move these to a secure config or build process in production
# For now, we will ask the user to input them or hardcode them here if provided.
# Since the user asked to SHOW a link to the bot, we need the bot username.
# And to SEND messages, we need the Bot Token.

# Placeholder - User must replace this
NOTIFICATION_BOT_TOKEN = "8217593955:AAGN4TSpuQcwGXclUDwniKeWaBDoUfEuvN4"
NOTIFICATION_BOT_USERNAME = "SoloveykoAINotificationBot" 
NOTIFICATION_BOT_URL = f"https://t.me/{NOTIFICATION_BOT_USERNAME}"

class NotificationManager:
    def __init__(self):
        pass

    def get_bot_url(self):
        return NOTIFICATION_BOT_URL

    def send_notification(self, message):
        """
        Sends a message to the configured Telegram User ID via the Bot.
        Runs in a separate thread to avoid blocking.
        """
        if not settings_manager.get("notifications_enabled", False):
            return

        user_id = settings_manager.get("telegram_user_id", "")
        if not user_id:
            logger.log("Notification skipped: No Telegram User ID configured.", level=LogLevel.WARNING)
            return

        token = NOTIFICATION_BOT_TOKEN
        
        if token == "YOUR_BOT_TOKEN_HERE":
             logger.log("Notification failed: Bot Token not configured in source code.", level=LogLevel.ERROR)
             return

        # Run in a separate thread to avoid blocking the main thread or worker threads
        # and to prevent QThreadStorage warnings related to request's thread-local data
        import threading
        thread = threading.Thread(target=self._send_request_thread, args=(token, user_id, message))
        thread.daemon = True
        thread.start()

    def _send_request_thread(self, token, user_id, message):
        url = f"https://api.telegram.org/bot{token}/sendMessage"
        payload = {
            "chat_id": user_id,
            "text": message
        }

        try:
            # logger.log(f"Sending Telegram notification to {user_id}...", level=LogLevel.INFO)
            response = requests.post(url, json=payload, timeout=10)
            response.raise_for_status()
            # logger.log("Telegram notification sent successfully.", level=LogLevel.SUCCESS)
        except requests.exceptions.RequestException as e:
             # Log only errors to avoid cluttering the log with success messages from background threads
            logger.log(f"Failed to send Telegram notification: {e}", level=LogLevel.ERROR)

    def send_test_notification(self):
        from utils.translator import translator
        message = translator.translate('notification_test_message')
        self.send_notification(message)

notification_manager = NotificationManager()
