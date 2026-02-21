package api

import (
	"context"
	"fmt"
	"time"

	"github.com/difyz9/edge-tts-go/pkg/communicate"
	"github.com/difyz9/edge-tts-go/pkg/voices"
)

type EdgeTTSService struct {
	OnLog func(level string, message string, details ...string)
}

func NewEdgeTTSService() *EdgeTTSService {
	return &EdgeTTSService{}
}

// Synthesize generates audio from text using Microsoft Edge TTS
func (s *EdgeTTSService) Synthesize(text string, voiceName string, rate string, pitch string, volume string, outputPath string, id string, taskLabel string) error {
	if rate == "" {
		rate = "+0%"
	}
	if pitch == "" {
		pitch = "+0Hz"
	}
	if volume == "" {
		volume = "+0%"
	}

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()

	// NewCommunicate(text, voice, rate, volume, pitch, proxy, connectTimeout, receiveTimeout, boundary)
	comm, err := communicate.NewCommunicate(text, voiceName, rate, volume, pitch, "", 10, 60)
	if err != nil {
		return fmt.Errorf("failed to create Edge TTS communicator: %v", err)
	}

	err = comm.Save(ctx, outputPath, "")
	if err != nil {
		return fmt.Errorf("failed to synthesize Edge TTS: %v", err)
	}

	return nil
}

type EdgeTTSVoice struct {
	Name           string `json:"Name"`
	ShortName      string `json:"ShortName"`
	Gender         string `json:"Gender"`
	Locale         string `json:"Locale"`
	SuggestedCodec string `json:"SuggestedCodec"`
	FriendlyName   string `json:"FriendlyName"`
}

func (s *EdgeTTSService) GetVoices() ([]EdgeTTSVoice, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	vlist, err := voices.ListVoices(ctx, "")
	if err != nil {
		return nil, fmt.Errorf("failed to fetch Edge TTS voices: %v", err)
	}

	var voices []EdgeTTSVoice
	for _, v := range vlist {
		voices = append(voices, EdgeTTSVoice{
			Name:         v.Name,
			ShortName:    v.ShortName,
			Gender:       v.Gender,
			Locale:       v.Locale,
			FriendlyName: v.FriendlyName,
		})
	}

	return voices, nil
}
