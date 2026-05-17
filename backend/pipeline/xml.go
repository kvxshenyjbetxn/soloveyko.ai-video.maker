package pipeline

import (
	"encoding/xml"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"
)

// XMEML v5 (FCP7 / DaVinci Resolve XML) struct definitions.

type xmemlDoc struct {
	XMLName  xml.Name `xml:"xmeml"`
	Version  string   `xml:"version,attr"`
	Sequence xmemlSeq `xml:"sequence"`
}

type xmemlSeq struct {
	Name     string     `xml:"name"`
	Duration int        `xml:"duration"`
	Rate     xmemlRate  `xml:"rate"`
	Media    xmemlMedia `xml:"media"`
}

type xmemlRate struct {
	Timebase int    `xml:"timebase"`
	NTSC     string `xml:"ntsc"`
}

type xmemlMedia struct {
	Video xmemlVideoMedia `xml:"video"`
	Audio xmemlAudioMedia `xml:"audio"`
}

type xmemlVideoMedia struct {
	Format xmemlVideoFormat `xml:"format"`
	Tracks []xmemlTrack     `xml:"track"`
}

type xmemlVideoFormat struct {
	SampleChars xmemlSampleChars `xml:"samplecharacteristics"`
}

type xmemlSampleChars struct {
	Width  int       `xml:"width"`
	Height int       `xml:"height"`
	Rate   xmemlRate `xml:"rate"`
}

type xmemlAudioMedia struct {
	Tracks []xmemlTrack `xml:"track"`
}

type xmemlTrack struct {
	Enabled string          `xml:"enabled"`
	Locked  string          `xml:"locked"`
	Clips   []xmemlClipItem `xml:"clipitem"`
}

type xmemlClipItem struct {
	ID       string        `xml:"id,attr"`
	Name     string        `xml:"name"`
	Enabled  string        `xml:"enabled"`
	Rate     *xmemlRate    `xml:"rate,omitempty"`
	Duration int           `xml:"duration"`
	Start    int           `xml:"start"`
	End      int           `xml:"end"`
	In       int           `xml:"in"`
	Out      int           `xml:"out"`
	File     *xmemlFile    `xml:"file,omitempty"`
	Motion   *xmemlMotion  `xml:"motion,omitempty"`
	Filters  []xmemlFilter `xml:"filter,omitempty"`
}

type xmemlFile struct {
	ID       string         `xml:"id,attr"`
	Name     string         `xml:"name,omitempty"`
	PathURL  string         `xml:"pathurl,omitempty"`
	Rate     *xmemlRate     `xml:"rate"`
	Duration int            `xml:"duration,omitempty"`
	Media    *xmemlFileMeta `xml:"media,omitempty"`
}

type xmemlFileMeta struct {
	Video *xmemlFileVideo `xml:"video,omitempty"`
	Audio *struct{}       `xml:"audio,omitempty"`
}

type xmemlFileVideo struct {
	SampleChars xmemlSampleChars `xml:"samplecharacteristics"`
}

type xmemlMotion struct {
	Params []xmemlParam `xml:"parameter"`
}

type xmemlParam struct {
	Name     string `xml:"name,omitempty"`
	ParamID  string `xml:"parameterid"`
	Value    string `xml:"value"`
	ValueMin string `xml:"valuemin,omitempty"`
	ValueMax string `xml:"valuemax,omitempty"`
}

type xmemlFilter struct {
	Enabled string      `xml:"enabled,omitempty"`
	Start   *int        `xml:"start,omitempty"`
	End     *int        `xml:"end,omitempty"`
	Effect  xmemlEffect `xml:"effect"`
}

type xmemlEffect struct {
	Name           string       `xml:"name,omitempty"`
	EffectID       string       `xml:"effectid"`
	EffectType     string       `xml:"effecttype,omitempty"`
	MediaType      string       `xml:"mediatype,omitempty"`
	EffectCategory string       `xml:"effectcategory,omitempty"`
	Params         []xmemlParam `xml:"parameter"`
}

// Data types used by GenerateFCPXML.

type xmlExportPlan struct {
	AudioPath      string
	AudioDuration  float64
	BaseW, BaseH   int
	Clips          []xmlClip
	Watermarks     []xmlWatermark
	Triggers       []xmlTrigger
	IntroPath      string
	IntroDuration  float64
	MainTrackOnTop bool    // if true, main clips track is added last (highest track number)
	ImageScale     float64 // масштаб V1-кліпів у %; 0 або 100 = без змін
	ScaleTarget    string  // "v1", "overlay", "all"
}

type xmlClip struct {
	Path     string
	Duration float64
	IsVideo  bool
}

type xmlWatermark struct {
	Path          string
	StartTime     float64
	Duration      float64
	X, Y          int
	W, H          int
	Opacity       float64
	TrackID       string
	IsVideo       bool
	ScaleOverride float64 // якщо > 0 і != 100, замінює позиційний масштаб
}

type xmlTrigger struct {
	Path      string
	StartTime float64
	Duration  float64
	X, Y      int
	W, H      int
	IsVideo   bool
}

// GenerateFCPXML writes an XMEML v5 file compatible with DaVinci Resolve and Final Cut Pro.
//
// Track layout:
//
//	V1 – intro (if any) + main clips sequential
//	V2..Vn – watermarks grouped by TrackID (order of first appearance)
//	V(n+1) – triggers
//	A1 – audio
func GenerateFCPXML(plan xmlExportPlan, fps int, outputPath string) error {
	toF := func(secs float64) int {
		return int(math.Round(secs * float64(fps)))
	}
	// DaVinci Resolve does NOT decode percent-encoded Cyrillic characters in pathurl,
	// but it DOES require spaces to be encoded as %20. Use a manual approach: build
	// "file://" + raw path with only spaces replaced by %20.
	pathToURL := func(p string) string {
		p = filepath.ToSlash(filepath.Clean(p))
		if !strings.HasPrefix(p, "/") {
			p = "/" + p // Windows: C:/... → /C:/...
		}
		return "file://" + strings.ReplaceAll(p, " ", "%20")
	}

	rate := xmemlRate{Timebase: fps, NTSC: "FALSE"}
	videoSC := xmemlSampleChars{Width: plan.BaseW, Height: plan.BaseH, Rate: rate}

	// Build media descriptor for a file element (only on first occurrence).
	buildMedia := func(isAudio, isVideo bool) *xmemlFileMeta {
		if isAudio {
			a := struct{}{}
			return &xmemlFileMeta{Audio: &a}
		}
		if isVideo {
			return &xmemlFileMeta{Video: &xmemlFileVideo{SampleChars: videoSC}}
		}
		// Still image — described as video media so DaVinci can place it on a video track.
		return &xmemlFileMeta{Video: &xmemlFileVideo{SampleChars: videoSC}}
	}

	// File registry: first reference carries full metadata, later ones just the id attr.
	fileByPath := make(map[string]string)
	nextID := 1
	getFile := func(path string, dur int, isAudio, isVideo bool) *xmemlFile {
		if fid, ok := fileByPath[path]; ok {
			return &xmemlFile{ID: fid}
		}
		fid := fmt.Sprintf("f%d", nextID)
		nextID++
		fileByPath[path] = fid
		r := rate
		// FCP7 standard: still images have no native duration; -1 signals "infinite" to editors.
		fileDur := dur
		if !isAudio && !isVideo {
			fileDur = -1
		}
		return &xmemlFile{
			ID:       fid,
			Name:     filepath.Base(path),
			PathURL:  pathToURL(path),
			Rate:     &r,
			Duration: fileDur,
			Media:    buildMedia(isAudio, isVideo),
		}
	}

	// V1: intro then main clips placed sequentially.
	var v1Clips []xmemlClipItem
	cursor := 0

	newTrack := func(clips []xmemlClipItem) xmemlTrack {
		return xmemlTrack{Enabled: "TRUE", Locked: "FALSE", Clips: clips}
	}
	newClip := func(id, name string, dur, start int, file *xmemlFile, isAudio bool) xmemlClipItem {
		item := xmemlClipItem{
			ID: id, Name: name, Enabled: "TRUE",
			Duration: dur, Start: start, End: start + dur, In: 0, Out: dur,
			File: file,
		}
		if !isAudio {
			r := rate
			item.Rate = &r
		}
		return item
	}

	applyToV1 := (plan.ScaleTarget == "v1" || plan.ScaleTarget == "all") &&
		plan.ImageScale > 0 && plan.ImageScale != 100

	if plan.IntroPath != "" && plan.IntroDuration > 0 {
		dur := toF(plan.IntroDuration)
		clip := newClip("clip-intro", filepath.Base(plan.IntroPath), dur, cursor,
			getFile(plan.IntroPath, dur, false, true), false)
		if applyToV1 {
			clip.Filters = append(clip.Filters, buildBasicMotionFilter(plan.ImageScale))
		}
		v1Clips = append(v1Clips, clip)
		cursor += dur
	}

	for i, c := range plan.Clips {
		if c.Path == "" || c.Duration <= 0 {
			continue
		}
		dur := toF(c.Duration)
		clip := newClip(fmt.Sprintf("clip-%d", i), filepath.Base(c.Path), dur, cursor,
			getFile(c.Path, dur, false, c.IsVideo), false)
		if applyToV1 {
			clip.Filters = append(clip.Filters, buildBasicMotionFilter(plan.ImageScale))
		}
		v1Clips = append(v1Clips, clip)
		cursor += dur
	}

	var videoTracks []xmemlTrack
	if !plan.MainTrackOnTop {
		videoTracks = append(videoTracks, newTrack(v1Clips))
	}

	// V2..Vn (or V1..Vn-1 if MainTrackOnTop): watermarks, one track per unique TrackID.
	trackOrder := []string{}
	trackItems := make(map[string][]xmemlClipItem)
	for i, wm := range plan.Watermarks {
		if wm.Path == "" || wm.Duration <= 0 {
			continue
		}
		if _, ok := trackItems[wm.TrackID]; !ok {
			trackOrder = append(trackOrder, wm.TrackID)
		}
		st := toF(wm.StartTime)
		dur := toF(wm.Duration)
		item := newClip(fmt.Sprintf("wm-%d", i), filepath.Base(wm.Path), dur, st,
			getFile(wm.Path, dur, false, wm.IsVideo), false)
		if wm.W > 0 && wm.H > 0 && plan.BaseW > 0 && plan.BaseH > 0 {
			item.Motion = buildXMLMotion(wm.X, wm.Y, wm.W, wm.H, plan.BaseW, plan.BaseH)
		}
		if wm.ScaleOverride > 0 && wm.ScaleOverride != 100 {
			item.Filters = append(item.Filters, buildBasicMotionFilter(wm.ScaleOverride))
		}
		if wm.Opacity > 0 && wm.Opacity < 1.0 {
			item.Filters = append(item.Filters, xmemlFilter{Effect: xmemlEffect{
				Name: "Opacity", EffectID: "opacity",
				Params: []xmemlParam{{ParamID: "opacity", Value: fmt.Sprintf("%.0f", wm.Opacity*100)}},
			}})
		}
		trackItems[wm.TrackID] = append(trackItems[wm.TrackID], item)
	}
	for _, tid := range trackOrder {
		videoTracks = append(videoTracks, newTrack(trackItems[tid]))
	}
	if plan.MainTrackOnTop {
		videoTracks = append(videoTracks, newTrack(v1Clips))
	}

	// V(n+1): triggers.
	var trigClips []xmemlClipItem
	for i, tr := range plan.Triggers {
		if tr.Path == "" || tr.Duration <= 0 {
			continue
		}
		st := toF(tr.StartTime)
		dur := toF(tr.Duration)
		item := newClip(fmt.Sprintf("trig-%d", i), filepath.Base(tr.Path), dur, st,
			getFile(tr.Path, dur, false, tr.IsVideo), false)
		if tr.W > 0 && tr.H > 0 && plan.BaseW > 0 && plan.BaseH > 0 {
			item.Motion = buildXMLMotion(tr.X, tr.Y, tr.W, tr.H, plan.BaseW, plan.BaseH)
		}
		trigClips = append(trigClips, item)
	}
	if len(trigClips) > 0 {
		videoTracks = append(videoTracks, newTrack(trigClips))
	}

	// A1: audio.
	totalDur := cursor
	if totalDur == 0 {
		totalDur = toF(plan.AudioDuration)
	}
	var audioTracks []xmemlTrack
	if plan.AudioPath != "" {
		audioDurF := toF(plan.AudioDuration)
		if audioDurF == 0 {
			audioDurF = totalDur
		}
		audioClip := newClip("audio-main", filepath.Base(plan.AudioPath), audioDurF, 0,
			getFile(plan.AudioPath, audioDurF, true, false), true)
		ar := rate
		audioClip.Rate = &ar
		audioTracks = []xmemlTrack{newTrack([]xmemlClipItem{audioClip})}
	}

	doc := xmemlDoc{
		Version: "5",
		Sequence: xmemlSeq{
			Name:     "Soloveyko Timeline",
			Duration: totalDur,
			Rate:     rate,
			Media: xmemlMedia{
				Video: xmemlVideoMedia{
					Format: xmemlVideoFormat{SampleChars: videoSC},
					Tracks: videoTracks,
				},
				Audio: xmemlAudioMedia{Tracks: audioTracks},
			},
		},
	}

	out, err := xml.MarshalIndent(doc, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal xmeml: %w", err)
	}
	content := xml.Header + "<!DOCTYPE xmeml>\n" + string(out)
	return os.WriteFile(outputPath, []byte(content), 0644)
}

// buildBasicMotionFilter returns a DaVinci-compatible Basic Motion filter with the given scale.
// DaVinci reads scale from <filter><effect id="basic">, not from the <motion> tag.
func buildBasicMotionFilter(scale float64) xmemlFilter {
	start, end := 0, 3600001
	return xmemlFilter{
		Enabled: "TRUE",
		Start:   &start,
		End:     &end,
		Effect: xmemlEffect{
			Name:           "Basic Motion",
			EffectID:       "basic",
			EffectType:     "motion",
			MediaType:      "video",
			EffectCategory: "motion",
			Params: []xmemlParam{
				{Name: "Scale", ParamID: "scale", Value: fmt.Sprintf("%.0f", scale), ValueMin: "0", ValueMax: "10000"},
			},
		},
	}
}

// buildXMLMotion converts pixel coordinates (top-left origin) to FCP7 motion parameters.
// scale: average scaling percentage; center: pixel offset from frame center (FCP Y is up).
func buildXMLMotion(x, y, w, h, baseW, baseH int) *xmemlMotion {
	scale := (float64(w)/float64(baseW) + float64(h)/float64(baseH)) / 2.0 * 100.0
	cx := float64(x+w/2) - float64(baseW)/2.0
	cy := float64(baseH)/2.0 - float64(y+h/2)
	return &xmemlMotion{Params: []xmemlParam{
		{ParamID: "scale", Value: fmt.Sprintf("%.2f", scale)},
		{ParamID: "center", Value: fmt.Sprintf("%.2f %.2f", cx, cy)},
	}}
}
