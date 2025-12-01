# Gemini Project: Multimedia Content Generation Tool

## Project Overview

This project is a desktop application for generating multimedia content, including translated text, voiceovers, and images. It provides a graphical user interface (GUI) built with PySide6 that allows users to create and manage content generation jobs. The application leverages several external APIs for its core functionality:

*   **OpenRouter API:** Used for text translation and image prompt generation.
*   **Googler API & Pollinations API:** Used for generating images from text prompts.
*   **ElevenLabs API:** Used for generating voiceovers from text.

The application is highly configurable, with settings stored in a `config/settings.json` file. These settings include API keys, language-specific prompts, and image generation parameters.

## Building and Running

This is a Python project that requires `PySide6`. To run the application, follow these steps:

1.  **Install Dependencies:**
    ```bash
    pip install -r requirements.txt
    ```

2.  **Configure the Application:**
    *   Open the `config/settings.json` file.
    *   Enter your API keys for OpenRouter, Googler, and ElevenLabs.
    *   Configure the language settings, models, and other parameters as needed.

3.  **Run the Application:**
    ```bash
    python main.py
    ```

## Development Conventions

*   **GUI:** The user interface is built using the PySide6 framework. The main window is structured with a tabbed interface to separate different functionalities (text processing, queue management, gallery, settings, and logs).
*   **Task Processing:** A queue-based system is used to manage content generation jobs. The `TaskProcessor` class processes jobs from the `QueueManager` in a separate thread pool to keep the UI responsive.
*   **API Integration:** Each external API is encapsulated in its own class within the `api/` directory. These classes handle the details of making API requests and processing the responses.
*   **Settings Management:** The `SettingsManager` class provides a centralized way to manage application settings, which are stored in a JSON file.
*   **Internationalization:** The application supports multiple languages using a `Translator` class that loads translations from JSON files in the `assets/translations/` directory.
*   **Styling:** The application uses a custom stylesheet based on `qt_material` to provide a modern look and feel. Themes are configurable in the settings.
