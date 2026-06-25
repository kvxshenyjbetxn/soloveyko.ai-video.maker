# Оновлення звіту кількості рядків коду

Команда рахує тільки Rust-файли в папці `src`:

```bash
python - <<'PY'
from pathlib import Path
from datetime import datetime

root = Path('.').resolve()
src = root / 'src'
report = root / 'loc-report.txt'

files = []
for path in src.rglob('*.rs'):
    if not path.is_file():
        continue

    try:
        text = path.read_text(encoding='utf-8-sig')
    except UnicodeDecodeError:
        text = path.read_text(encoding='utf-16')

    lines = 0 if text == '' else len(text.splitlines())
    files.append((path.relative_to(root).as_posix(), lines))

files.sort(key=lambda item: (-item[1], item[0].lower()))
total = sum(lines for _, lines in files)

with report.open('w', encoding='utf-8', newline='\n') as out:
    out.write('LOC report: src/**/*.rs\n')
    out.write(f'Generated: {datetime.now().isoformat(timespec="seconds")}\n')
    out.write(f'Root: {root}\n')
    out.write('Scope: only .rs files inside src/\n')
    out.write('Note: physical line count; blank/comment lines are included.\n')
    out.write('\n')
    out.write(f'TOTAL FILES: {len(files)}\n')
    out.write(f'TOTAL LINES: {total}\n')
    out.write('\n')
    out.write(f'{"Lines":>8}  File\n')
    out.write('-' * 90 + '\n')
    for rel, lines in files:
        out.write(f'{lines:>8}  {rel}\n')

print(f'Updated {report.name}')
print(f'TOTAL FILES: {len(files)}')
print(f'TOTAL LINES: {total}')
PY
```

Після виконання оновиться файл `loc-report.txt` у корені проєкту.
