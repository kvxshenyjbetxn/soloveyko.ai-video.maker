import json
import os
import platform
import sys
import copy
from PySide6.QtCore import QStandardPaths

class SettingsManager:
    def __init__(self, settings_file='config/settings.json'):
        self.base_path = self._get_base_path()
        self.settings_file = os.path.join(self.base_path, settings_file)
        self.defaults = {
            'language': 'uk',
            'theme': 'dark',
            'results_path': self._get_default_results_path(),
            'image_review_enabled': True,
            'rewrite_review_enabled': True,
            'translation_review_enabled': True,
            'prompt_count_control_enabled': True,
            'prompt_count': 50,
            'detailed_logging_enabled': True,
            'voicemaker_api_key': '',
            'voicemaker_char_limit': 2900,
            'gemini_tts_api_key': '',
            'gemini_tts_url': 'https://gemini-tts-server-beta-production.up.railway.app',
            'image_generation_provider': 'pollinations',
            'image_generation_prompt': "",
            'image_prompt_settings': {
                'prompt': """Role: You are an expert AI Prompt Engineer specializing in creating image generation prompts for Imagen 4. Your task is to analyze the provided story (regardless of genre: sci-fi, nature, history, fantasy) and create EXACTLY 50 high-quality photorealistic prompts.

**CRITICAL RULE: NO PROPER NAMES.**
You must NEVER use the names of characters (e.g., "John," "Rex," "Apollo 11"). Instead, you must create a "Visual Description Token" for each subject (e.g., "a tall man with a beard," "a small rusty robot," "a spotted jaguar") and use that description consistently.

**STEP 1: IDENTIFY THE HOOK**
Find the HOOK section of the story (the engaging intro).
* It is the first 10-15% of the text designed to grab attention.
* The first 5 prompts MUST be based ONLY on this hook section.

**STEP 2: SUBJECT & ENTITY ANALYSIS (UNIVERSAL)**
Read the story and identify the visual subjects. Since this can be ANY niche, adapt your analysis:
* **Main Entities:** Can be Humans, Animals, Aliens, Robots, Vehicles, or Abstract concepts.
    * *Define:* Physical appearance, texture (skin/fur/metal), colors, size, distinctive features.
* **Setting:** Time period, location (space/forest/city), lighting, atmosphere.
* **Create Visual Tokens:** Replace names with these descriptions (e.g., "The Commander" -> "A stern woman in a grey naval uniform").

**STEP 3: SCENE DISTRIBUTION**
The story must be divided as follows:
* **Prompts 1-5:** HOOK section ONLY.
* **Prompts 6-50:** Main story (45 prompts evenly distributed).
    * 6-15: Beginning
    * 16-25: Early middle
    * 26-35: Middle
    * 36-45: Late middle
    * 46-50: Ending

**STEP 4: UNIFIED STYLE DEFINITION**
Define ONE unified photorealistic style for all images:
* **Universal Realism:** "Photorealistic, looks like a real photograph."
* **Texture Quality:** "Realistic skin textures" (for humans), "Realistic fur details" (for animals), "Realistic metal weathering" (for machines).
* **Cinematography:** "Natural lighting, shot on professional 35mm camera, 8k resolution."
* **NO TEXT:** "No text, no subtitles, no captions, no words."

**RULES FOR CREATING PROMPTS:**
1.  **Subject Consistency:** Use the EXACT same physical description for the same subject in every prompt. NEVER use their name.
2.  **Scene Selection:** Ensure scenes follow the chronological plot.
3.  **Forbidden Terms:** NEVER use: cartoon, animated, illustration, drawing, painting, anime, sketch.
4.  **Safety:** No blood, violence, or disturbing imagery.
5.  **Formatting:** Each prompt in English. Use only commas. Number 1 to 50.

**OUTPUT FORMAT REQUIREMENTS (STRICT):**
* **NO META-COMMENTARY:** Do not write "Hook identified," "Character list," "Here are the prompts."
* **DIRECT OUTPUT:** Start immediately with "1. Photorealistic photograph..."
* **COUNT:** Exactly 50 prompts.

**TEMPLATE:**
1. Photorealistic photograph, [scene from hook], [Subject Description NO NAME], [Setting], natural lighting, realistic textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality
2. Photorealistic photograph, ...
...
50. Photorealistic photograph, [final scene], [Subject Description NO NAME], [Setting], ...

**TASK:**
Read the text below. Identify the hook. Create generic subject descriptions (removing all names). Generate EXACTLY 50 photorealistic prompts.

STORY TEXT:

""",
                'model': 'z-ai/glm-4.5-air:free',
                'max_tokens': 0,
                'temperature': 1.0
            },
            'preview_settings': {
                'model': 'z-ai/glm-4.5-air:free',
                'max_tokens': 0,
                'temperature': 1.0,
                'image_count': 3,
                'prompt': """Act as a professional Prompt Engineer and YouTube Thumbnail Expert. Analyze the VIDEO TITLE and STORY provided at the bottom of this message. Based on the text, generate 3 (three) highly detailed image generation prompts for Midjourney/DALL-E 3. STRICT GUIDELINES FOR THE OUTPUT PROMPTS: 1. LANGUAGE: Output prompts must be in English. 2. NO TEXT: The generated images must NOT contain any text, typography, letters, watermarks, or logos. 3. NO PROPER NAMES: Do not use names. Replace them with visual descriptions (e.g., "an astronaut," "a giant asteroid," "a detective"). 4. SAFETY: Ensure descriptions are PG-13 and safe, but keep them dramatic. 5. COMPOSITION: The image MUST show the interaction between the Protagonist and the Antagonist (or the main Object/Mystery/Entity) described in the story. They must be visually connected in the frame to create harmony and storytelling. 6. CLICKBAIT GRAPHICS: You MUST include specific instructions to overlay graphical elements. Use phrases like: - "Overlay a bright red arrow pointing at..." - "A thick red circle graphic drawn around..." 7. STYLE: Photorealistic, 8k, Cinematic lighting, Unreal Engine 5 render, dramatic atmosphere, YouTube thumbnail style. OUTPUT STRUCTURE (Provide 3 distinct options): Option 1: "Conflict & Emotion" (Focus on facial expressions and tension + Red Arrow pointing to the main reaction or threat). Option 2: "Mystery & Atmosphere" (Focus on the environment/silhouettes + Red Circle around a mysterious figure/object/anomaly). Option 3: "The Key Detail" (Focus on the specific hook mentioned in the title, e.g., a strange planet, artifact, or clue + Red Circle AND Arrow pointing to it).Do not write "I am ready" or any conversational text. Just output the 3 prompts. Write each prompt separately and numbered, nothing else, do not write anything like “Option 3:” --- INPUT DATA: VIDEO TITLE: {title} STORY: {story}"""
            },
            'googler': {
                'api_key': '',
                'aspect_ratio': 'IMAGE_ASPECT_RATIO_LANDSCAPE',
                'max_threads': 25,
                'max_video_threads': 10,
                'video_prompt': 'Slow dolly shot, natural movement of all elements, realistic physics and motion blur, ambient environmental audio, atmospheric lighting with subtle changes, organic details animating naturally, cinematic quality with smooth transitions, no text overlay, no subtitles.',
                'seed': '',
                'negative_prompt': 'low quality, worst quality, low resolution, blurry, pixelated, jpeg artifacts, grainy, noise, distorted, deformation, bad anatomy, bad proportions, disconnected limbs, extra limbs, extra fingers, missing fingers, floating limbs, mutated hands, poorly drawn face, disfigured, ugly, asymmetry, text, watermark, signature, logo, username, artist name, copyright, timestamp, cropped, out of frame, cut off, bad composition, overexposed, underexposed, flat lighting, dull colors, oversaturated'
            },
            'pollinations': {
                'model': 'zimage',
                'token': '',
                'width': 1920,
                'height': 1080,
                'nologo': True,
                'enhance': False
            },
            'openrouter_api_key': '',
            'ai_assistant_model': 'openai/gpt-4o-mini',
            'openrouter_models': ['z-ai/glm-4.5-air:free', 'google/gemini-2.5-flash', 'openai/gpt-4o-mini'],
            'subtitles': {
                'whisper_model': 'base',
                'font': 'Impact',
                'fontsize': 100,
                'color': [255, 255, 255],
                'fade_in': 150,
                'fade_out': 150,
                'margin_v': 100,
                'max_words': 10
            },
            'montage': {
                'preset': 'superfast',
                'bitrate_mbps': 5,
                'upscale_factor': 2,
                'transition_duration': 2,
                'enable_sway': True,
                'max_concurrent_montages': 1
            },
            'languages_config': {
                'uk': {
                    'display_name': 'Ukraine',
                    'prompt': """Role: You are an expert literary translator and cultural editor specializing in "transculturation" (total cultural adaptation).

Task: Analyze the provided source text (input language may vary), retain the core plot, emotions, and character archetypes, but perform a deep localization to rewrite the story as if it is taking place entirely in Ukraine.

Goal: The final text must read like an original Ukrainian work, written for Ukrainians, with NO foreign cultural markers remaining.

**ADAPTATION PROTOCOLS:**

**1. TOTAL RELOCATION (Geography & Setting):**
* **Analyze the Setting:** Identify the physical environment of the source (e.g., mountains, dense forest, metropolis, rural plains, desert, coastal city).
* **Map to Ukraine:** Relocate the action to the closest equivalent Ukrainian region.
    * *Examples:* High mountains → Carpathians; Deep forests/swamps → Polissia; Industrial city → Kryvyi Rih/Dnipro/Donbas context; Cultural capital → Lviv/Kyiv; Seaside → Odesa/Black Sea coast; Steppe/Fields → Kherson/Central Ukraine.
* **Nature:** Replace foreign flora/fauna with Ukrainian equivalents (e.g., palm trees → poplars/chestnuts; rattlesnakes → vipers; moose → elk/bison).

**2. NAMES & CHARACTERS:**
* **Ukrainize Names:** Replace ALL foreign names with natural Ukrainian equivalents that match the character's age and vibe (e.g., John → Ivan; Jessica → Yulia/Oksana; Mr. Smith → Pan Kovalenko).
* **Forms of Address:** Use "Pan/Pani" or first names/patronymics depending on the social context. Remove "Sir", "Monsieur", "Mister", etc.

**3. CULTURAL & SOCIAL ELEMENTS:**
* **Institutions:** Replace foreign agencies (FBI, Scotland Yard, Sheriff) with Ukrainian realities (SBU, National Police, Dilnychyi).
* **Daily Life:** Replace food, drinks, clothing, and brands with those familiar to a Ukrainian (e.g., whiskey → horilka/brandy/samohon; burger → varenyky/deruny/kotleta; baseball → football).
* **Measurement:** Convert all units to Metric (miles → km, Fahrenheit → Celsius).

**4. LANGUAGE & STYLE:**
* **No Calques:** Completely abandon the source language syntax.
* **Idioms:** Replace foreign metaphors with authentic Ukrainian proverbs and sayings.
* **Euphony:** Ensure the text has "mylozvuchnist" (flows beautifully in Ukrainian).

**OUTPUT FORMAT REQUIREMENTS (STRICT):**
* **NO META-COMMENTARY:** Do NOT write "Here is the translation," "I have adapted the text," or any introductory remarks.
* **DIRECT OUTPUT:** Begin immediately with the first sentence of the adapted story.
* **TARGET LANGUAGE:** Ukrainian only.

Original Source Text:

""",
                    'model': 'z-ai/glm-4.5-air:free',
                    'max_tokens': 0,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert Ukrainian documentary scriptwriter and historian (similar to the style of high-quality channels like "Історія без міфів" or top-tier dubbed documentaries).

Task: Analyze the provided source text (input language may vary), extract the core facts, events, and atmosphere, and synthesize a completely new narrative script in Ukrainian.

Goal: Create an immersive, "edutainment" style narrative. Do not translate word-for-word. Instead, reimagine the content as if it were originally written by a talented Ukrainian narrator for a Ukrainian audience.

Rewrite Protocols (Universal Input):
1.  **Analyze & Extract:** Read the source text to understand the "What", "Where", "When", and "Why".
2.  **Discard Source Syntax:** Completely ignore the sentence structure of the original text. Do not mimic foreign phrasing (avoid English SVO structures or Russian syntax patterns).
3.  **Authentic Ukrainian:** Use natural Ukrainian idioms, rich vocabulary, and correct syntax. Ensure "euphony" (милозвучність) – the text must flow smoothly when read aloud.
4.  **Terminology:** Use correct Ukrainian historical terminology and spelling.

Style Guidelines:
* **Cinematic & Sensory:** Describe the atmosphere, architecture, and mood.
* **Engagement:** Use a "hook" at the beginning. Address the audience (e.g., "Уявіть собі...", "Мало хто знає...").
* **Tone:** Fascinating, professional, yet accessible.

Output Format Requirements (STRICT):
* **NO META-COMMENTARY:** Do not write "Here is the Ukrainian version," "Gart," or any intro.
* **IMMEDIATE START:** Begin directly with the first sentence of the story.
* **TARGET LANGUAGE:** Ukrainian only.

Original Source Text:

""",
                    'rewrite_model': 'z-ai/glm-4.5-air:free',
                    'rewrite_max_tokens': 0,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'uk-UA-OstapNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                },
                'en': {
                    'display_name': 'English',
                    'prompt': """Role: You are an expert literary translator and cultural editor specializing in "transculturation" (total cultural adaptation).

Task: Analyze the provided source text, retain the core plot, emotions, and character archetypes, but perform a deep localization to rewrite the story as if it is taking place entirely in an English-speaking Western context (e.g., USA, UK, or Canada, depending on the best fit for the atmosphere).

Goal: The final text must read like an original English-language work, written for a Western audience, with NO foreign cultural markers remaining.

ADAPTATION PROTOCOLS:

1. TOTAL RELOCATION (Geography & Setting):

Analyze the Setting: Identify the physical environment of the source (e.g., mountains, dense forest, metropolis, rural plains, steppe, coastal city).

Map to the West: Relocate the action to a logical equivalent.

Examples: High mountains → Rockies/Appalachians; Deep forests → Pacific Northwest/Maine; Industrial city → Rust Belt/Chicago/Manchester; Cultural capital → NYC/London; Seaside → California coast/Cornwall; Steppe/Fields → Midwest/Great Plains.

Nature: Replace foreign flora/fauna with local equivalents (e.g., poplars → oaks/pines; vipers → rattlesnakes; local specific berries → blueberries/cranberries).

2. NAMES & CHARACTERS:

Westernize Names: Replace ALL foreign names with natural English-equivalent names that match the character's age, social status, and vibe (e.g., Ivan → John; Oksana → Jane/Claire; Pan Kovalenko → Mr. Smith).

Forms of Address: Use "Mr./Ms.", "Sir", or first names depending on the social context. Remove any foreign honorifics (e.g., Pan, San, Monsieur).

3. CULTURAL & SOCIAL ELEMENTS:

Institutions: Replace foreign agencies with local realities (e.g., SBU → FBI/MI5; Police structures → Sheriff/Local PD).

Daily Life: Replace food, drinks, clothing, and brands with those familiar to a Westerner (e.g., horilka → whiskey/bourbon; varenyky → pierogies/dumplings or simply replace with burgers/steak; local folk music → country/rock/jazz).

Measurement: Convert all units to the Imperial system for US contexts (km → miles, Celsius → Fahrenheit) or keep Metric for UK/Canadian contexts if appropriate.

4. LANGUAGE & STYLE:

No Calques: Completely abandon the source language syntax and sentence structure.

Idioms: Replace foreign metaphors and proverbs with authentic English idioms and colloquialisms.

Flow: Ensure the text has a natural, "native" narrative rhythm.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do NOT write "Here is the translation," "I have adapted the text," or any introductory remarks.

DIRECT OUTPUT: Begin immediately with the first sentence of the adapted story.

TARGET LANGUAGE: English only.

Original Source Text:

""",
                    'model': 'z-ai/glm-4.5-air:free',
                    'max_tokens': 0,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert documentary scriptwriter and historian (specializing in the style of high-quality narrative media like BBC, National Geographic, or the Vox "Explained" series).

Task: Analyze the provided source text (input language may vary), extract the core facts, events, and atmosphere, and synthesize a completely new narrative script in English.

Goal: Create an immersive, "edutainment" style narrative. Do not translate word-for-word. Instead, reimagine the content as if it were originally written by a world-class English-speaking screenwriter for a global audience.

REWRITE PROTOCOLS:

1. Analyze & Extract: * Deeply process the "What", "Where", "When", and "Why" of the source.

Identify the key emotional beats and historical significance.

2. Discard Source Syntax: * Completely ignore the original sentence structure.

Avoid any phrasing that feels like a translation. The result must feel natively English in its rhythm, punchiness, and flow.

3. Narrative Sophistication: * Use rich, evocative vocabulary and compelling rhetorical devices.

Ensure the script is optimized for voiceover (it must sound natural and impactful when read aloud).

4. Contextual Nuance: * Use precise historical and technical terminology recognized in the English-speaking academic and media world.

STYLE GUIDELINES:

Cinematic & Sensory: Vividly describe the setting, the atmosphere of the era, and the sensory details (architecture, sounds, weather).

Engagement: Start with a strong "hook." Use narrative techniques to draw the audience in (e.g., "Imagine a world where...", "Few realize that...", "It was a moment that changed everything...").

Tone: Authoritative yet captivating; professional yet accessible.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do NOT include introductory remarks like "Sure, here is the script" or "I have rewritten the text."

IMMEDIATE START: Begin directly with the first sentence of the story or narrator's line.

TARGET LANGUAGE: English only.

Original Source Text:

""",
                    'rewrite_model': 'z-ai/glm-4.5-air:free',
                    'rewrite_max_tokens': 0,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'en-US-AndrewNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                },
                'ru': {
                    'display_name': 'Russian',
                    'prompt': """Role: You are an expert literary translator and cultural editor specializing in "transculturation" (total cultural adaptation).

Task: Analyze the provided source text (input language may vary), retain the core plot, emotions, and character archetypes, but perform a deep localization to rewrite the story as if it is taking place entirely in Russia.

Goal: The final text must read like an original Russian work, written for Russians, with NO foreign cultural markers remaining.

ADAPTATION PROTOCOLS:

1. TOTAL RELOCATION (Geography & Setting):

Analyze the Setting: Identify the physical environment of the source (e.g., mountains, dense forest, metropolis, rural plains, desert, coastal city).

Map to Russia: Relocate the action to the closest equivalent Russian region.

Examples: High mountains → Caucasus/Altai; Deep forests/swamps → Taiga/Siberia/Central Russia; Industrial city → Chelyabinsk/Norilsk/Ural context; Cultural capital → St. Petersburg/Moscow; Seaside → Sochi/Crimea/Black Sea coast; Steppe/Fields → Southern Russia/Kuban.

Nature: Replace foreign flora/fauna with Russian equivalents (e.g., palm trees → birches/pines; rattlesnakes → vipers; moose → elk/bear).

2. NAMES & CHARACTERS:

Russianize Names: Replace ALL foreign names with natural Russian equivalents that match the character's age and vibe (e.g., John → Ivan; Jessica → Yulia/Oksana; Mr. Smith → Gospodin/Pan Kovalenko/Sidorov).

Forms of Address: Use natural Russian address forms (Name + Patronymic for formal, or informal diminutives). Remove "Sir", "Monsieur", "Mister", etc.

3. CULTURAL & SOCIAL ELEMENTS:

Institutions: Replace foreign agencies (FBI, Scotland Yard, Sheriff) with Russian realities (FSB, MVD, Politsiya/Uchastkovyi).

Daily Life: Replace food, drinks, clothing, and brands with those familiar to a Russian (e.g., whiskey → vodka/samogon; burger → pirozhok/pelmeni/kotleta; baseball → football/hockey).

Measurement: Convert all units to Metric (miles → km, Fahrenheit → Celsius).

4. LANGUAGE & STYLE:

No Calques: Completely abandon the source language syntax.

Idioms: Replace foreign metaphors with authentic Russian proverbs and sayings.

Flow: Ensure the text has natural Russian "flavor" and linguistic richness.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do NOT write "Here is the translation," "I have adapted the text," or any introductory remarks.

DIRECT OUTPUT: Begin immediately with the first sentence of the adapted story.

TARGET LANGUAGE: Russian only.

Original Source Text:

""",
                    'model': 'z-ai/glm-4.5-air:free',
                    'max_tokens': 0,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert Russian documentary scriptwriter and historian (similar to the style of high-quality channels like "Redaktsiya", "Minayev Live", or top-tier dubbed documentaries).

Task: Analyze the provided source text (input language may vary), extract the core facts, events, and atmosphere, and synthesize a completely new narrative script in Russian.

Goal: Create an immersive, "edutainment" style narrative. Do not translate word-for-word. Instead, reimagine the content as if it were originally written by a talented Russian narrator for a Russian audience.

REWRITE PROTOCOLS (Universal Input):

Analyze & Extract: Read the source text to understand the "What", "Where", "When", and "Why".

Discard Source Syntax: Completely ignore the sentence structure of the original text. Do not mimic foreign phrasing (avoid English SVO structures or specific non-Russian syntax patterns).

Authentic Russian: Use natural Russian idioms, rich vocabulary, and correct syntax. Ensure the text flows smoothly when read aloud (voiceover-ready).

Terminology: Use correct Russian historical terminology and academic spelling.

STYLE GUIDELINES:

Cinematic & Sensory: Describe the atmosphere, architecture, and mood.

Engagement: Use a "hook" at the beginning. Address the audience (e.g., "Представьте себе...", "Мало кто знает...").

Tone: Fascinating, professional, yet accessible.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do not write "Here is the Russian version," or any intro.

IMMEDIATE START: Begin directly with the first sentence of the story.

TARGET LANGUAGE: Russian only.

Original Source Text:

""",
                    'rewrite_model': 'z-ai/glm-4.5-air:free',
                    'rewrite_max_tokens': 0,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'ru-RU-DmitryNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                },
                'pl': {
                    'display_name': 'Poland',
                    'prompt': """Role: You are an expert literary translator and cultural editor specializing in "transculturation" (total cultural adaptation).

Task: Analyze the provided source text (input language may vary), retain the core plot, emotions, and character archetypes, but perform a deep localization to rewrite the story as if it is taking place entirely in Poland.

Goal: The final text must read like an original Polish work, written for Poles, with NO foreign cultural markers remaining.

ADAPTATION PROTOCOLS:

1. TOTAL RELOCATION (Geography & Setting):

Analyze the Setting: Identify the physical environment of the source (e.g., mountains, dense forest, metropolis, rural plains, desert, coastal city).

Map to Poland: Relocate the action to the closest equivalent Polish region.

Examples: High mountains → Tatry; Deep forests/swamps → Białowieża/Mazury; Industrial city → Śląsk (Katowice/Gliwice); Cultural capital → Kraków/Warszawa; Seaside → Gdańsk/Bałtyk; Rural plains → Mazowsze/Wielkopolska.

Nature: Replace foreign flora/fauna with Polish equivalents (e.g., palm trees → wierzby/dęby; rattlesnakes → żmije; moose → łoś/żubr).

2. NAMES & CHARACTERS:

Polishize Names: Replace ALL foreign names with natural Polish equivalents that match the character's age and vibe (e.g., John → Jan/Janek; Jessica → Kasia/Małgorzata; Mr. Smith → Pan Kowalski).

Forms of Address: Use "Pan/Pani" or natural Polish honorifics. Remove "Sir", "Monsieur", "Mister", etc.

3. CULTURAL & SOCIAL ELEMENTS:

Institutions: Replace foreign agencies (FBI, Scotland Yard, Sheriff) with Polish realities (ABW, Policja, Straż Miejska).

Daily Life: Replace food, drinks, clothing, and brands with those familiar to a Pole (e.g., whiskey → wódka/nalewka; burger → pierogi/schabowy; baseball → piłka nożna/siatkówka).

Measurement: Convert all units to Metric (miles → km, Fahrenheit → Celsius).

4. LANGUAGE & STYLE:

No Calques: Completely abandon the source language syntax.

Idioms: Replace foreign metaphors with authentic Polish proverbs and sayings (e.g., "mądry Polak po szkodzie").

Flow: Ensure the text has natural Polish flow and linguistic beauty.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do NOT write "Here is the translation," "I have adapted the text," or any introductory remarks.

DIRECT OUTPUT: Begin immediately with the first sentence of the adapted story.

TARGET LANGUAGE: Polish only.

Original Source Text:

""",
                    'model': 'z-ai/glm-4.5-air:free',
                    'max_tokens': 0,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert Polish documentary scriptwriter and historian (similar to the style of high-quality narrative channels like "Historyczny Top", "Ciekawehistorie", or top-tier documentaries).

Task: Analyze the provided source text (input language may vary), extract the core facts, events, and atmosphere, and synthesize a completely new narrative script in Polish.

Goal: Create an immersive, "edutainment" style narrative. Do not translate word-for-word. Instead, reimagine the content as if it were originally written by a talented Polish narrator for a Polish audience.

REWRITE PROTOCOLS (Universal Input):

Analyze & Extract: Read the source text to understand the "What", "Where", "When", and "Why".

Discard Source Syntax: Completely ignore the sentence structure of the original text. Do not mimic foreign phrasing (avoid English SVO structures or specific non-Polish syntax patterns).

Authentic Polish: Use natural Polish idioms, rich vocabulary, and correct syntax. Ensure the text flows smoothly when read aloud (voiceover-ready).

Terminology: Use correct Polish historical terminology and academic spelling.

STYLE GUIDELINES:

Cinematic & Sensory: Describe the atmosphere, architecture, and mood.

Engagement: Use a "hook" at the beginning. Address the audience (e.g., "Wyobraźcie sobie...", "Mało kto wie, że...").

Tone: Fascinating, professional, yet accessible.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do not write "Oto polska wersja," or any intro.

IMMEDIATE START: Begin directly with the first sentence of the story.

TARGET LANGUAGE: Polish only.

Original Source Text:

""",
                    'rewrite_model': 'z-ai/glm-4.5-air:free',
                    'rewrite_max_tokens': 0,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'pl-PL-MarekNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                },
                'ro': {
                    'display_name': 'Romania',
                    'prompt': """Role: You are an expert literary translator and cultural editor specializing in "transculturation" (total cultural adaptation).

Task: Analyze the provided source text (input language may vary), retain the core plot, emotions, and character archetypes, but perform a deep localization to rewrite the story as if it is taking place entirely in Romania.

Goal: The final text must read like an original Romanian work, written for Romanians, with NO foreign cultural markers remaining.

ADAPTATION PROTOCOLS:

1. TOTAL RELOCATION (Geography & Setting):

Analyze the Setting: Identify the physical environment of the source (e.g., mountains, dense forest, metropolis, rural plains, desert, coastal city).

Map to Romania: Relocate the action to the closest equivalent Romanian region.

Examples: High mountains → Carpathians (Carpati); Deep forests → Transylvania/Maramureș; Industrial city → Ploiești/Galați/Reșița; Cultural capital → Bucharest/Cluj-Napoca/Iași; Seaside → Constanța/Black Sea coast; Steppe/Fields → Bărăgan/Dobrogea.

Nature: Replace foreign flora/fauna with Romanian equivalents (e.g., palm trees → tei/salcâmi; rattlesnakes → vipere; moose → zimbru/urs).

2. NAMES & CHARACTERS:

Romanianize Names: Replace ALL foreign names with natural Romanian equivalents that match the character's age and vibe (e.g., John → Ion/Andrei; Jessica → Ioana/Elena; Mr. Smith → Domnul Popescu/Ionescu).

Forms of Address: Use "Domnul/Doamna" or natural Romanian honorifics. Remove "Sir", "Monsieur", "Mister", etc.

3. CULTURAL & SOCIAL ELEMENTS:

Institutions: Replace foreign agencies (FBI, Scotland Yard, Sheriff) with Romanian realities (SRI, Poliția Română, Jandarmeria).

Daily Life: Replace food, drinks, clothing, and brands with those familiar to a Romanian (e.g., whiskey → țuică/pălincă; burger → mici/sarmale/mămăligă; baseball → fotbal/oină).

Measurement: Convert all units to Metric (miles → km, Fahrenheit → Celsius).

4. LANGUAGE & STYLE:

No Calques: Completely abandon the source language syntax.

Idioms: Replace foreign metaphors with authentic Romanian proverbs and sayings (e.g., "cu o floare nu se face primăvară").

Flow: Ensure the text has natural Romanian flow and linguistic richness.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do NOT write "Here is the translation," "I have adapted the text," or any introductory remarks.

DIRECT OUTPUT: Begin immediately with the first sentence of the adapted story.

TARGET LANGUAGE: Romanian only.

Original Source Text:

""",
                    'model': 'z-ai/glm-4.5-air:free',
                    'max_tokens': 0,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert Romanian documentary scriptwriter and historian (similar to the style of high-quality narrative channels like "Zaiafet", "Istoria cu Virgil", or top-tier documentaries).

Task: Analyze the provided source text (input language may vary), extract the core facts, events, and atmosphere, and synthesize a completely new narrative script in Romanian.

Goal: Create an immersive, "edutainment" style narrative. Do not translate word-for-word. Instead, reimagine the content as if it were originally written by a talented Romanian narrator for a Romanian audience.

REWRITE PROTOCOLS (Universal Input):

Analyze & Extract: Read the source text to understand the "What", "Where", "When", and "Why".

Discard Source Syntax: Completely ignore the sentence structure of the original text. Do not mimic foreign phrasing (avoid English SVO structures or specific non-Romanian syntax patterns).

Authentic Romanian: Use natural Romanian idioms, rich vocabulary, and correct syntax. Ensure the text flows smoothly when read aloud (voiceover-ready).

Terminology: Use correct Romanian historical terminology and academic spelling.

STYLE GUIDELINES:

Cinematic & Sensory: Describe the atmosphere, architecture, and mood.

Engagement: Use a "hook" at the beginning. Address the audience (e.g., "Imaginați-vă...", "Puțini sunt cei care știu că...").

Tone: Fascinating, professional, yet accessible.

OUTPUT FORMAT REQUIREMENTS (STRICT):

NO META-COMMENTARY: Do not write "Iată versiunea în română," or any intro.

IMMEDIATE START: Begin directly with the first sentence of the story.

TARGET LANGUAGE: Romanian only.

Original Source Text:

""",
                    'rewrite_model': 'z-ai/glm-4.5-air:free',
                    'rewrite_max_tokens': 0,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'ro-RO-EmilNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                }
            },
            'custom_stages': [],
            'last_used_template_name': '',
            'notifications_enabled': False,
            'telegram_user_id': '',
            'show_welcome_dialog': True
        }
        self.settings = self.load_settings()

    def _get_default_results_path(self):
        try:
            # Спробуємо отримати стандартну папку для Відео/Фільмів через Qt
            path = QStandardPaths.writableLocation(QStandardPaths.StandardLocation.MoviesLocation)
            
            # Якщо Qt не повернув шлях (наприклад, викликано занадто рано), 
            # спробуємо стандартні шляхи ОС
            if not path:
                home = os.path.expanduser("~")
                if platform.system() == "Darwin":
                    path = os.path.join(home, "Movies")
                elif platform.system() == "Windows":
                    # Спробуємо знайти папку Відео через змінну середовища або стандартний шлях
                    path = os.path.join(os.environ.get('USERPROFILE', home), 'Videos')
                else:
                    path = os.path.join(home, "Videos")
            
            # Додаємо назву нашої програми
            default_path = os.path.join(path, "Soloveyko.AI-Video.Maker")
            
            # Не створюємо папку тут, вона буде створена при запуску задачі (TaskState)
            return default_path
        except Exception as e:
            print(f"DEBUG: Error calculating default results path: {e}")
            return os.path.expanduser("~/Soloveyko.AI-Video.Maker-Results")

    def _get_base_path(self):
        if platform.system() == "Darwin":
            # На macOS використовуємо Application Support для уникнення PermissionError
            app_support = os.path.expanduser("~/Library/Application Support/Soloveyko.AI-Video.Maker")
            
            try:
                os.makedirs(app_support, exist_ok=True)
            except Exception as e:
                print(f"DEBUG: Error creating directory {app_support}: {e}")
            
            # Створюємо симлінк у Документах для видимості
            docs_link = os.path.expanduser("~/Documents/Soloveyko.AI-Video.Maker")
            if not os.path.exists(docs_link):
                try:
                    # Символьне посилання дозволяє бачити папку в Документах
                    os.symlink(app_support, docs_link)
                    print(f"DEBUG: Created symlink in Documents: {docs_link}")
                except Exception as e:
                    print(f"DEBUG: Could not create symlink in Documents: {e}")
                    # Це не критично, програма продовжить працювати через App Support
            
            print(f"DEBUG: macOS Data Path is: {app_support}")
            return app_support
            
        if getattr(sys, 'frozen', False):
            # Поруч з EXE (Windows)
            return os.path.dirname(sys.executable)
        else:
            # Корень проекту (батьківська папка utils)
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def load_settings(self):
        print(f"DEBUG: Loading settings from {self.settings_file}")
        
        current_settings = {}
        if os.path.exists(self.settings_file):
            # Список кодувань для спроби
            encodings = ['utf-8-sig', 'utf-8', 'utf-16', 'cp1251']
            
            for enc in encodings:
                try:
                    with open(self.settings_file, 'r', encoding=enc) as f:
                        loaded = json.load(f)
                        if isinstance(loaded, dict):
                            current_settings = loaded
                            print(f"DEBUG: Successfully loaded settings with {enc}")
                            break
                        else:
                            print(f"DEBUG: Loaded settings is not a dictionary ({type(loaded)})")
                except (json.JSONDecodeError, UnicodeDecodeError):
                    continue 
                except Exception as e:
                    print(f"DEBUG: Error loading settings with {enc}: {e}")
                    break
        else:
            print("DEBUG: Settings file does not exist, using defaults.")

        # РЕКУРСИВНО заповнюємо відсутні ключі з дефолтів
        self._deep_merge(self.defaults, current_settings)
        
        # Якщо результати порожні, ставимо дефолтний шлях
        if not current_settings.get('results_path'):
            current_settings['results_path'] = self.defaults.get('results_path')
            
        return current_settings

    def _deep_merge(self, source, destination, is_languages_root=False):
        """
        Рекурсивно копіює ключі з source в destination, якщо їх там немає.
        Спеціальна логіка для languages_config: не додаємо відсутні мови, якщо розділ існує,
        щоб дозволити користувачеві видаляти стандартні мови.
        """
        for key, value in source.items():
            if isinstance(value, dict):
                if key not in destination or not isinstance(destination[key], dict):
                    # Якщо ми всередині languages_config і цієї мови немає в налаштуваннях користувача - пропускаємо її
                    if is_languages_root:
                        continue
                    destination[key] = copy.deepcopy(value)
                else:
                    self._deep_merge(value, destination[key], is_languages_root=(key == 'languages_config'))
            else:
                if key not in destination:
                    destination[key] = value
        return destination

    def get(self, key, default=None):
        if "." in key:
            parts = key.split(".")
            val = self.settings
            try:
                for p in parts:
                    val = val[p]
                return val
            except (KeyError, TypeError):
                 # Fallback to defaults
                 val = self.defaults
                 try:
                    for p in parts:
                        val = val[p]
                    return val
                 except (KeyError, TypeError):
                    return default

        if default is None:
            default = self.defaults.get(key)
        return self.settings.get(key, default)

    def set(self, key, value):
        if "." in key:
            parts = key.split(".")
            target = self.settings
            for p in parts[:-1]:
                if p not in target or not isinstance(target[p], dict):
                    target[p] = {}
                target = target[p]
            target[parts[-1]] = value
        else:
            self.settings[key] = value
        self.save_settings()

    def save_settings(self):
        try:
            os.makedirs(os.path.dirname(self.settings_file), exist_ok=True)
            with open(self.settings_file, 'w', encoding='utf-8') as f:
                json.dump(self.settings, f, indent=4, ensure_ascii=False)
                f.flush()
                os.fsync(f.fileno())
        except Exception as e:
            print(f"Error saving settings: {e}")

class TemplateManager:
    def __init__(self, template_dir='config/templates'):
        self.base_path = self._get_base_path()
        self.template_dir = os.path.join(self.base_path, template_dir)
        os.makedirs(self.template_dir, exist_ok=True)

    def _get_base_path(self):
        if platform.system() == "Darwin":
            return os.path.expanduser("~/Library/Application Support/Soloveyko.AI-Video.Maker")

        if getattr(sys, 'frozen', False):
            return os.path.dirname(sys.executable)
        else:
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def get_templates(self):
        templates = [f.split('.')[0] for f in os.listdir(self.template_dir) if f.endswith('.json')]
        return sorted(templates)

    def load_template(self, name):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        if os.path.exists(file_path):
            with open(file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        return {}

    def save_template(self, name, data):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=4, ensure_ascii=False)

    def delete_template(self, name):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        if os.path.exists(file_path):
            os.remove(file_path)

    def rename_template(self, old_name, new_name):
        old_path = os.path.join(self.template_dir, f"{old_name}.json")
        new_path = os.path.join(self.template_dir, f"{new_name}.json")
        if os.path.exists(old_path) and not os.path.exists(new_path):
            os.rename(old_path, new_path)

settings_manager = SettingsManager()
template_manager = TemplateManager()
