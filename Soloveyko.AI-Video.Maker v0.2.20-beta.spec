# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[('assets/ffmpeg.exe', 'assets'), ('assets/ffprobe.exe', 'assets'), ('assets/icon.ico', 'assets'), ('assets/icon.png', 'assets'), ('assets/icon.icns', 'assets'), ('assets/gemini_tts_voices.json', 'assets'), ('assets/voicemaker_voices.json', 'assets'), ('assets/translations', 'assets/translations'), ('assets/styles', 'assets/styles'), ('gui/qt_material', 'gui/qt_material'), ('C:\\Users\\kvxshenyjbetxn\\AppData\\Local\\Programs\\Python\\Python313\\Lib\\site-packages\\whisper\\assets', 'whisper/assets')],
    hiddenimports=['whisper'],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=['PyQt6', 'PyQt5', 'tkinter'],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='Soloveyko.AI-Video.Maker v0.2.20-beta',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=['assets\\icon.ico'],
)
