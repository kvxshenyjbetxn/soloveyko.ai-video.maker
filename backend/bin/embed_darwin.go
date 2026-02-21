//go:build darwin

package bin

import "embed"

//go:embed ffmpeg ffprobe whisper
var Files embed.FS
