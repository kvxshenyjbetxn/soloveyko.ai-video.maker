//go:build darwin

package bin

import "embed"

//go:embed ffmpeg ffprobe whisper exiftool_mac.zip
var Files embed.FS
