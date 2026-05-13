package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync/atomic"
	"time"

	"soloveyko/backend/mcpserver"

	wruntime "github.com/wailsapp/wails/v2/pkg/runtime"
)

type agentActionResponse struct {
	Payload json.RawMessage
	Error   string
}

func (a *App) AgentControllerReady() {
	if a.agentReadyCh == nil {
		a.agentReadyCh = make(chan struct{})
	}
	a.agentReadyOnce.Do(func() {
		close(a.agentReadyCh)
		a.startLocalMCPController()
	})
}

func (a *App) ResolveAgentRequest(id string, payload string, errorText string) {
	if id == "" {
		return
	}

	a.agentPendingMu.Lock()
	ch, ok := a.agentPending[id]
	if ok {
		delete(a.agentPending, id)
	}
	a.agentPendingMu.Unlock()
	if !ok {
		return
	}

	response := agentActionResponse{Error: errorText}
	if payload != "" {
		response.Payload = json.RawMessage(payload)
	}

	select {
	case ch <- response:
	default:
	}
}

func (a *App) GetLocalMCPStatus() map[string]interface{} {
	if a.mcpController == nil {
		return map[string]interface{}{
			"enabled": false,
		}
	}

	status := a.mcpController.Status()
	return map[string]interface{}{
		"enabled":   status.Enabled,
		"transport": status.Transport,
		"address":   status.Address,
		"endpoint":  status.Endpoint,
		"server":    status.Server,
		"version":   status.Version,
	}
}

func (a *App) startLocalMCPController() {
	if a.mcpController != nil {
		return
	}

	controller, err := mcpserver.Start(func(ctx context.Context, action string, params any) (json.RawMessage, error) {
		return a.invokeAgentAction(ctx, action, params)
	}, nil)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[MCP] Failed to start local MCP server: %v", err))
		return
	}

	a.mcpController = controller
	status := controller.Status()
	a.LogToUI("INFO", fmt.Sprintf("[MCP] Local MCP server ready: %s", status.Endpoint))
}

func (a *App) invokeAgentAction(ctx context.Context, action string, params any) (json.RawMessage, error) {
	if a.ctx == nil {
		return nil, errors.New("application UI context is not ready")
	}
	if a.agentPending == nil {
		a.agentPending = make(map[string]chan agentActionResponse)
	}
	if a.agentReadyCh == nil {
		a.agentReadyCh = make(chan struct{})
	}

	waitCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	select {
	case <-a.agentReadyCh:
	case <-waitCtx.Done():
		return nil, errors.New("agent controller is not ready")
	}

	requestID := fmt.Sprintf("agent_%d", atomic.AddUint64(&a.agentReqCounter, 1))
	responseCh := make(chan agentActionResponse, 1)

	a.agentPendingMu.Lock()
	a.agentPending[requestID] = responseCh
	a.agentPendingMu.Unlock()

	wruntime.EventsEmit(a.ctx, "agent:request", map[string]interface{}{
		"id":     requestID,
		"action": action,
		"params": params,
	})

	select {
	case response := <-responseCh:
		if response.Error != "" {
			return nil, errors.New(response.Error)
		}
		return response.Payload, nil
	case <-ctx.Done():
		a.agentPendingMu.Lock()
		delete(a.agentPending, requestID)
		a.agentPendingMu.Unlock()
		return nil, ctx.Err()
	}
}
