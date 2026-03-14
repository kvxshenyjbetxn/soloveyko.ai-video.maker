---
trigger: always_on
---

СИСТЕМНА ІНФОРМАЦІЯ: Ти знаходишся на Windows 11/10. ВСІ команди виконуються ТІЛЬКИ через PowerShell (pwsh.exe або powershell.exe). 

ОСНОВНІ ПРАВИЛА КОМАНД:
- НІКОЛИ не використовуй bash, sh, apt, curl (якщо не через winget), git bash синтаксис
- PowerShell команди: Get-ChildItem (НЕ ls), Remove-Item (НЕ rm), Copy-Item (НЕ cp) 
- Шляхи: C:\Users\...\ (з \), НЕ /home/user/
- Виконання: powershell.exe -Command "команда" або pwsh -c "команда"
- Завантаження: Invoke-WebRequest -Uri URL -OutFile file.exe (НЕ curl | bash)
- Процеси: Get-Process, Stop-Process (НЕ ps, kill)
- FFmpeg/Golang: .\ffmpeg.exe, go mod tidy (Windows шляхи)

ПЕРЕВІРКА ПЕРЕД ВІДПОВІДДЮ:
1. Якщо генеруєш команду - перевір: чи це PowerShell синтаксис?
2. Якщо користувач не уточнив ОС - РОБИ Windows/PowerShell за замовчуванням
3. Показуй повну команду: powershell.exe -ExecutionPolicy Bypass -Command "твоя команда"

ПРИКЛАДИ:
❌ НЕПРАВИЛЬНО: sudo apt install ffmpeg; cd /home/
✅ ПРАВИЛЬНО: winget install ffmpeg; cd C:\Projects\
❌ НЕПРАВИЛЬНО: chmod +x script.sh; ./script.sh  
✅ ПРАВИЛЬНО: .\script.ps1 або Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy Bypass; .\script.ps1
