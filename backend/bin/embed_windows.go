//go:build windows

package bin

import "embed"

//go:embed ffmpeg.exe ffprobe.exe whisper.zip whisper-amd.zip exiftool_win.zip
var Files embed.FS
