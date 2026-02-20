package engines

import "embed"

// Binaries містить вбудовані бінарні файли (ffprobe)
// Вона буде автоматично запакована в Go-бінарник.
// Додайте ffprobe.exe (Windows) та ffprobe (macOS) у цю папку.
//
//go:embed *
var Binaries embed.FS
