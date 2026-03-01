export type SettingType = 'select' | 'input' | 'number' | 'switch' | 'slider' | 'path' | 'color';

export interface MassEditorSetting {
    id: string;
    labelKey: string;
    type: SettingType;
    options?: { value: any; label: string }[];
    min?: number;
    max?: number;
    step?: number;
    dynamicModels?: 'openrouter' | 'pollinations' | 'voicemaker' | 'elevenlabsbot' | 'edgetts' | 'edgetts-r' | 'edgetts-p' | 'edgetts-v';
    dynamicKeys?: 'openrouter' | 'elevenlabsbot' | 'elevenlabsua' | 'elevenlabsunlim' | 'voicemaker' | 'pollinations' | 'elevenlabsimage';
    path?: string;
}

export interface MassEditorBlock {
    id: string;
    labelKey: string;
    settings: MassEditorSetting[];
}

export const MASS_EDITOR_BLOCKS: MassEditorBlock[] = [
    {
        id: 'api',
        labelKey: 'settings.api',
        settings: [
            { id: 'translateOpenRouterKeyID', labelKey: 'settings.api_keys.openrouter', type: 'select', dynamicKeys: 'openrouter', path: 'api.translateOpenRouterKeyID' },
            { id: 'voiceoverElevenLabsBotKeyID', labelKey: 'settings.api_keys.elevenlabs_bot', type: 'select', dynamicKeys: 'elevenlabsbot', path: 'api.voiceoverElevenLabsBotKeyID' },
            { id: 'voiceoverElevenLabsUnlimKeyID', labelKey: 'settings.api_keys.elevenlabs_unlim', type: 'select', dynamicKeys: 'elevenlabsunlim', path: 'api.voiceoverElevenLabsUnlimKeyID' },
            { id: 'voiceoverElevenLabsUAKeyID', labelKey: 'settings.api_keys.elevenlabs_ua', type: 'select', dynamicKeys: 'elevenlabsua', path: 'api.voiceoverElevenLabsUAKeyID' },
            { id: 'voiceoverVoiceMakerKeyID', labelKey: 'settings.api_keys.voicemaker', type: 'select', dynamicKeys: 'voicemaker', path: 'api.voiceoverVoiceMakerKeyID' },
            { id: 'imagePollinationsKeyID', labelKey: 'settings.api_keys.pollinations', type: 'select', dynamicKeys: 'pollinations', path: 'api.imagePollinationsKeyID' },
            { id: 'elevenLabsImageKeyID', labelKey: 'settings.api_keys.elevenlabs_image', type: 'select', dynamicKeys: 'elevenlabsimage', path: 'api.elevenLabsImageKeyID' },
        ],
    },
    {
        id: 'translate',
        labelKey: 'text.translate',
        settings: [
            { id: 'translateEnabled', labelKey: 'settings.text.enabled', type: 'switch', path: 'stages.translate' },
            { id: 'translateModel', labelKey: 'settings.text.translate_model', type: 'select', dynamicModels: 'openrouter', path: 'text.translateModel' },
            { id: 'translateTemperature', labelKey: 'settings.text.translate_temperature', type: 'slider', min: 0, max: 2, step: 0.1, path: 'text.translateTemperature' },
            { id: 'translateMaxTokens', labelKey: 'settings.text.translate_max_tokens', type: 'number', path: 'text.translateMaxTokens' },
            { id: 'translatePrompt', labelKey: 'settings.text.translate_prompt', type: 'input', path: 'text.translatePrompt' },
        ],
    },
    {
        id: 'rewrite',
        labelKey: 'text.rewrite',
        settings: [
            { id: 'rewriteEnabled', labelKey: 'settings.text.enabled', type: 'switch', path: 'stages.rewrite' },
            { id: 'rewriteModel', labelKey: 'settings.text.rewrite_model', type: 'select', dynamicModels: 'openrouter', path: 'text.rewriteModel' },
            { id: 'rewriteTemperature', labelKey: 'settings.text.rewrite_temperature', type: 'slider', min: 0, max: 2, step: 0.1, path: 'text.rewriteTemperature' },
            { id: 'rewriteMaxTokens', labelKey: 'settings.text.rewrite_max_tokens', type: 'number', path: 'text.rewriteMaxTokens' },
            { id: 'rewritePrompt', labelKey: 'settings.text.rewrite_prompt', type: 'input', path: 'text.rewritePrompt' },
        ],
    },
    {
        id: 'voiceover',
        labelKey: 'stages.voiceover',
        settings: [
            { id: 'voiceoverEnabled', labelKey: 'settings.voiceover.enabled', type: 'switch', path: 'stages.voiceover' },
            {
                id: 'voiceoverService', labelKey: 'settings.voiceover.service', type: 'select', options: [
                    { value: 'elevenlabsbot', label: 'ElevenLabs Bot' },
                    { value: 'elevenlabsunlim', label: 'ElevenLabs Unlim' },
                    { value: 'elevenlabsua', label: 'ElevenLabs UA' },
                    { value: 'voicemaker', label: 'VoiceMaker' },
                    { value: 'edgetts', label: 'EdgeTTS' },
                ], path: 'voiceover.voiceoverService'
            },
            { id: 'voiceoverTemplate', labelKey: 'pipeline.voiceover.elevenlabsbot.template', type: 'select', dynamicModels: 'elevenlabsbot', path: 'voiceover.services.elevenlabsbot.template' },
            // ElevenLabs Unlim
            { id: 'elevenLabsUnlimVoiceID', labelKey: 'pipeline.voiceover.elevenlabsunlim.voice_id', type: 'input', path: 'voiceover.services.elevenlabsunlim.voice_id' },
            { id: 'elevenLabsUnlimStability', labelKey: 'pipeline.voiceover.elevenlabsunlim.stability', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsunlim.stability' },
            { id: 'elevenLabsUnlimSimilarity', labelKey: 'pipeline.voiceover.elevenlabsunlim.similarity', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsunlim.similarity' },
            { id: 'elevenLabsUnlimStyle', labelKey: 'pipeline.voiceover.elevenlabsunlim.style', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsunlim.style' },
            { id: 'elevenLabsUnlimSpeakerBoost', labelKey: 'pipeline.voiceover.elevenlabsunlim.speaker_boost', type: 'switch', path: 'voiceover.services.elevenlabsunlim.speaker_boost' },
            // ElevenLabs UA
            { id: 'elevenLabsUAVoiceID', labelKey: 'pipeline.voiceover.elevenlabsua.voice_id', type: 'input', path: 'voiceover.services.elevenlabsua.voice_id' },
            { id: 'elevenLabsUAStability', labelKey: 'pipeline.voiceover.elevenlabsua.stability', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsua.stability' },
            { id: 'elevenLabsUASimilarity', labelKey: 'pipeline.voiceover.elevenlabsua.similarity', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsua.similarity' },
            { id: 'elevenLabsUAStyle', labelKey: 'pipeline.voiceover.elevenlabsua.style', type: 'slider', min: 0, max: 1, step: 0.01, path: 'voiceover.services.elevenlabsua.style' },
            { id: 'elevenLabsUASpeakerBoost', labelKey: 'pipeline.voiceover.elevenlabsua.speaker_boost', type: 'switch', path: 'voiceover.services.elevenlabsua.speaker_boost' },
            {
                id: 'elevenLabsUAModel', labelKey: 'pipeline.voiceover.elevenlabsua.model', type: 'select', options: [
                    { value: 'eleven_multilingual_v2', label: 'Multilingual v2' },
                    { value: 'eleven_flash_v2_5', label: 'Flash v2.5' },
                    { value: 'eleven_turbo_v2_5', label: 'Turbo v2.5' },
                    { value: 'eleven_multilingual_v3', label: 'v3 (Emotions)' },
                ], path: 'voiceover.services.elevenlabsua.model'
            },
            // VoiceMaker
            { id: 'voiceMakerVoiceID', labelKey: 'pipeline.voiceover.voicemaker.voice_id', type: 'select', dynamicModels: 'voicemaker', path: 'voiceover.services.voicemaker.voice_id' },
            { id: 'voiceMakerCharLimit', labelKey: 'pipeline.voiceover.voicemaker.char_limit', type: 'number', path: 'voiceover.services.voicemaker.char_limit' },
            // EdgeTTS
            { id: 'edgeTTSVoiceID', labelKey: 'pipeline.voiceover.edgetts.voice_id', type: 'select', dynamicModels: 'edgetts', path: 'voiceover.services.edgetts.voice_id' },
            { id: 'edgeTTSRate', labelKey: 'pipeline.voiceover.edgetts.rate', type: 'slider', min: -50, max: 50, step: 1, dynamicModels: 'edgetts-r', path: 'voiceover.services.edgetts.rate' },
            { id: 'edgeTTSPitch', labelKey: 'pipeline.voiceover.edgetts.pitch', type: 'slider', min: -50, max: 50, step: 1, dynamicModels: 'edgetts-p', path: 'voiceover.services.edgetts.pitch' },
            { id: 'edgeTTSVolume', labelKey: 'pipeline.voiceover.edgetts.volume', type: 'slider', min: -50, max: 50, step: 1, dynamicModels: 'edgetts-v', path: 'voiceover.services.edgetts.volume' },
        ],
    },
    {
        id: 'image',
        labelKey: 'stages.image',
        settings: [
            { id: 'imageEnabled', labelKey: 'settings.image.enabled', type: 'switch', path: 'stages.image' },
            {
                id: 'imageService', labelKey: 'settings.image.service', type: 'select', options: [
                    { value: 'pollinations', label: 'Pollinations' },
                    { value: 'googler', label: 'Googler' },
                    { value: 'elevenlabsimage', label: 'ElevenLabs Image' },
                ], path: 'image.imageService'
            },
            { id: 'imageModel', labelKey: 'settings.image.model', type: 'select', dynamicModels: 'pollinations', path: 'image.services.pollinations.imageModel' },
            { id: 'imageWidth', labelKey: 'settings.image.width', type: 'number', path: 'image.services.pollinations.imageWidth' },
            { id: 'imageHeight', labelKey: 'settings.image.height', type: 'number', path: 'image.services.pollinations.imageHeight' },
            { id: 'imagePrompt', labelKey: 'settings.image.prompt', type: 'input', path: 'image.services.pollinations.imagePrompt' },
            {
                id: 'imageGenerationMethod', labelKey: 'settings.image.generationMethod', type: 'select', options: [
                    { value: 'lines', label: 'By Lines' },
                    { value: 'sentences', label: 'By Sentences' },
                ], path: 'image.imageGenerationMethod'
            },
            { id: 'imageGroupSentences', labelKey: 'settings.image.group', type: 'switch', path: 'image.imageGroupSentences' },
            { id: 'imageInitialSentenceCount', labelKey: 'settings.image.initial_count', type: 'number', path: 'image.imageInitialSentenceCount' },
            { id: 'imageSentenceLimit', labelKey: 'settings.image.limit', type: 'number', path: 'image.imageSentenceLimit' },
            { id: 'imagePromptModel', labelKey: 'settings.image.prompt_model', type: 'select', dynamicModels: 'openrouter', path: 'image.imagePromptModel' },
            { id: 'imagePromptTemperature', labelKey: 'settings.image.prompt_temp', type: 'slider', min: 0, max: 2, step: 0.1, path: 'image.imagePromptTemperature' },
            { id: 'imagePromptMaxTokens', labelKey: 'settings.image.prompt_tokens', type: 'number', path: 'image.imagePromptMaxTokens' },
            {
                id: 'imageMode', labelKey: 'pipeline.image.mode', type: 'select', options: [
                    { value: 'normal', label: 'pipeline.image.mode_normal' },
                    { value: 'memory', label: 'pipeline.image.mode_memory' },
                ], path: 'image.imageMode'
            },
            {
                id: 'imageMemoryType', labelKey: 'pipeline.image.memory_type', type: 'select', options: [
                    { value: 'primitive', label: 'pipeline.image.memory_type_primitive' },
                    { value: 'external', label: 'pipeline.image.memory_type_external' },
                ], path: 'image.imageMemoryType'
            },
            { id: 'imageMemoryChars', labelKey: 'pipeline.image.memory_chars', type: 'number', path: 'image.imageMemoryChars' },
            { id: 'imageDetermineCharacters', labelKey: 'pipeline.image.determine_characters', type: 'switch', path: 'image.imageDetermineCharacters' },
            { id: 'imageDetermineCharactersPrompt', labelKey: 'pipeline.image.determine_characters_prompt', type: 'input', path: 'image.imageDetermineCharactersPrompt' },
            // Googler specific
            {
                id: 'imageGooglerModel', labelKey: 'settings.image.googler.model', type: 'select', options: [
                    { value: 'whisk', label: 'Whisk (v4)' },
                    { value: 'flow', label: 'Flow (v4)' },
                    { value: 'grok', label: 'Grok (v4)' },
                    { value: 'gemini', label: 'Gemini (v4)' },
                ], path: 'image.services.googler.imageGooglerModel'
            },
            { id: 'imageGooglerVideoEnabled', labelKey: 'settings.image.googler.video', type: 'switch', path: 'image.services.googler.imageGooglerVideoEnabled' },
            {
                id: 'imageGooglerAspectRatio', labelKey: 'pipeline.image.aspect_ratio', type: 'select', options: [
                    { value: 'IMAGE_ASPECT_RATIO_PORTRAIT', label: 'Portrait (9:16)' },
                    { value: 'IMAGE_ASPECT_RATIO_LANDSCAPE', label: 'Landscape (16:9)' },
                ], path: 'image.services.googler.imageGooglerAspectRatio'
            },
        ],
    },
    {
        id: 'subtitle',
        labelKey: 'stages.subtitles',
        settings: [
            { id: 'subtitleEnabled', labelKey: 'settings.subtitle.enabled', type: 'switch', path: 'stages.subtitle' },
            {
                id: 'subtitleModel', labelKey: 'settings.subtitle.model', type: 'select', options: [
                    { value: 'tiny', label: 'Tiny' },
                    { value: 'base', label: 'Base' },
                    { value: 'small', label: 'Small' },
                    { value: 'medium', label: 'Medium' },
                    { value: 'large-v1', label: 'Large-v1' },
                    { value: 'large-v2', label: 'Large-v2' },
                    { value: 'large-v3', label: 'Large-v3' },
                ], path: 'subtitle.subtitleModel'
            },
            { id: 'subtitleAmdLanguage', labelKey: 'settings.subtitle.amd.language', type: 'input', path: 'subtitle.subtitleAmdLanguage' },
            {
                id: 'subtitleFont', labelKey: 'settings.subtitle.font', type: 'select', options: [
                    'Arial', 'Montserrat', 'Inter', 'Roboto', 'Open Sans', 'Verdana', 'Tahoma',
                    'Impact', 'Georgia', 'Times New Roman', 'Courier New', 'Comic Sans MS',
                    'Trebuchet MS', 'Arial Black', 'Palatino', 'Garamond', 'Bookman', 'Avant Garde',
                    'Helvetica', 'Century Gothic', 'Futura', 'Gill Sans', 'Franklin Gothic',
                    'Candara', 'Calibri', 'Cambria', 'Constantia', 'Corbel', 'Segoe UI',
                    'Ubuntu', 'Noto Sans', 'Oswald', 'Raleway', 'Playfair Display',
                    'Poppins', 'Muli', 'Lato', 'Quicksand', 'Nunito', 'Karla', 'Lora',
                    'Bebas Neue', 'Source Sans Pro', 'Merriweather', 'PT Sans', 'PT Serif'
                ].sort().map(f => ({ value: f, label: f })), path: 'subtitle.subtitleFont'
            },
            { id: 'subtitleSize', labelKey: 'settings.subtitle.size', type: 'number', path: 'subtitle.subtitleSize' },
            { id: 'subtitleColor', labelKey: 'settings.subtitle.color', type: 'color', path: 'subtitle.subtitleColor' },
            { id: 'subtitleMaxLen', labelKey: 'settings.subtitle.max_len', type: 'number', path: 'subtitle.subtitleMaxLen' },
            { id: 'subtitleFadeEnabled', labelKey: 'settings.subtitle.fade_enabled', type: 'switch', path: 'subtitle.subtitleFadeEnabled' },
            { id: 'subtitleFadeIn', labelKey: 'settings.subtitle.fade_in', type: 'number', path: 'subtitle.subtitleFadeIn' },
            { id: 'subtitleFadeOut', labelKey: 'settings.subtitle.fade_out', type: 'number', path: 'subtitle.subtitleFadeOut' },
        ],
    },
    {
        id: 'montage',
        labelKey: 'stages.montage',
        settings: [
            { id: 'montageEnabled', labelKey: 'settings.montage.enabled', type: 'switch', path: 'stages.montage' },
            {
                id: 'montageResolution', labelKey: 'settings.montage.resolution', type: 'select', options: [
                    { value: '720p', label: '720p' },
                    { value: '1080p', label: '1080p' },
                    { value: '2k', label: '2k' },
                ], path: 'montage.montageResolution'
            },
            {
                id: 'montageFPS', labelKey: 'settings.montage.fps', type: 'select', options: [
                    { value: 24, label: '24' },
                    { value: 30, label: '30' },
                    { value: 60, label: '60' },
                ], path: 'montage.montageFPS'
            },
            { id: 'montageSwayFactor', labelKey: 'settings.montage.sway', type: 'slider', min: 0, max: 3, step: 0.1, path: 'montage.montageSwayFactor' },
            { id: 'montageZoomFactor', labelKey: 'settings.montage.zoom', type: 'slider', min: 0, max: 3, step: 0.1, path: 'montage.montageZoomFactor' },
            { id: 'montageUpscaleFactor', labelKey: 'settings.montage.internal_upscale', type: 'slider', min: 1, max: 3, step: 0.1, path: 'montage.montageUpscaleFactor' },
            { id: 'montageTransitionDuration', labelKey: 'settings.montage.transitions', type: 'slider', min: 0.1, max: 2, step: 0.05, path: 'montage.montageTransitionDuration' },
            {
                id: 'montageTransitionEffect', labelKey: 'settings.montage.transition_effect', type: 'select', options: [
                    "fade_fast", "fade", "wipeleft", "wiperight", "wipeup", "wipedown",
                    "slideleft", "slideright", "slideup", "slidedown", "circlecrop",
                    "rectcrop", "distance", "fadeblack", "fadewhite", "radial",
                    "smoothleft", "smoothright", "smoothup", "smoothdown",
                    "circleopen", "circleclose", "vertopen", "vertclose",
                    "horzopen", "horzclose", "dissolve", "pixelize", "diagtl",
                    "diagtr", "diagbl", "diagbr"
                ].map(effect => ({ value: effect, label: effect })), path: 'montage.montageTransitionEffect'
            },
            {
                id: 'montageEncodingPreset', labelKey: 'settings.montage.encoding_preset', type: 'select', options: [
                    { value: 'ultrafast', label: 'Ultrafast' },
                    { value: 'superfast', label: 'Superfast' },
                    { value: 'veryfast', label: 'Veryfast' },
                    { value: 'faster', label: 'Faster' },
                    { value: 'fast', label: 'Fast' },
                    { value: 'medium', label: 'Medium' },
                    { value: 'slow', label: 'Slow' },
                    { value: 'slower', label: 'Slower' },
                    { value: 'veryslow', label: 'Veryslow' },
                ], path: 'montage.montageEncodingPreset'
            },
            { id: 'montageBitrate', labelKey: 'settings.montage.bitrate', type: 'number', path: 'montage.montageBitrate' },
            // Watermark & Intro
            { id: 'montageIntroVideoEnabled', labelKey: 'settings.montage.intro_enabled', type: 'switch', path: 'montage.montageIntroVideoEnabled' },
            { id: 'montageIntroVideoPath', labelKey: 'settings.montage.intro_path', type: 'path', path: 'montage.montageIntroVideoPath' },
            { id: 'montageWatermarkEnabled', labelKey: 'settings.montage.watermark_enabled', type: 'switch', path: 'montage.montageWatermarkEnabled' },
            { id: 'montageWatermarkPath', labelKey: 'settings.montage.watermark_path', type: 'path', path: 'montage.montageWatermarkPath' },
            { id: 'montageWatermarkOpacity', labelKey: 'settings.montage.watermark_opacity', type: 'slider', min: 0, max: 1, step: 0.1, path: 'montage.montageWatermarkOpacity' },
            { id: 'montageWatermarkSize', labelKey: 'settings.montage.watermark_size', type: 'slider', min: 1, max: 50, step: 1, path: 'montage.montageWatermarkSize' },
            { id: 'montageOverlayEnabled', labelKey: 'settings.montage.overlay_enabled', type: 'switch', path: 'montage.montageOverlayEnabled' },
            { id: 'montageOverlayPath', labelKey: 'settings.montage.overlay_path', type: 'path', path: 'montage.montageOverlayPath' },
        ],
    },
    {
        id: 'control',
        labelKey: 'pipeline.group.control',
        settings: [
            { id: 'translateControlEnabled', labelKey: 'pipeline.translate_control', type: 'switch', path: 'control.translateControlEnabled' },
            { id: 'imageControlEnabled', labelKey: 'pipeline.image_control', type: 'switch', path: 'control.imageControlEnabled' },
        ],
    },
    {
        id: 'custom',
        labelKey: 'pipeline.group.custom_stages',
        settings: [
            { id: 'customStagesEnabled', labelKey: 'settings.text.enabled', type: 'switch', path: 'stages.customStagesEnabled' },
        ],
    },
    {
        id: 'paths',
        labelKey: 'pipeline.group.path',
        settings: [
            { id: 'outputPath', labelKey: 'settings.paths.output', type: 'path', path: 'paths.outputPath' },
        ],
    },
];
