package mcpserver

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"

	"soloveyko/backend/utils"
)

type Invoker func(ctx context.Context, action string, params any) (json.RawMessage, error)

type Status struct {
	Enabled   bool   `json:"enabled"`
	Transport string `json:"transport"`
	Address   string `json:"address"`
	Endpoint  string `json:"endpoint"`
	Server    string `json:"server"`
	Version   string `json:"version"`
}

type Server struct {
	httpServer *http.Server
	listener   net.Listener
	status     Status
}

type SetMainTextArgs struct {
	Tab      string `json:"tab,omitempty" jsonschema:"target text tab: translate or rewrite"`
	Text     string `json:"text" jsonschema:"text to write into the main draft editor"`
	FocusTab bool   `json:"focusTab,omitempty" jsonschema:"switch the UI to the chosen text tab"`
}

type GetMainTextArgs struct {
	Tab string `json:"tab,omitempty" jsonschema:"target text tab: translate or rewrite"`
}

type SelectTemplatesArgs struct {
	TaskType      string   `json:"taskType,omitempty" jsonschema:"template type to select: translate or rewrite"`
	TemplateIDs   []string `json:"templateIds,omitempty" jsonschema:"template ids to select"`
	TemplateNames []string `json:"templateNames,omitempty" jsonschema:"template names to select"`
}

type EnqueueTaskArgs struct {
	TaskType        string   `json:"taskType,omitempty" jsonschema:"pipeline type: translate or rewrite"`
	TaskName        string   `json:"taskName" jsonschema:"human-readable task name"`
	Text            string   `json:"text,omitempty" jsonschema:"optional text override; defaults to the current main draft"`
	TemplateIDs     []string `json:"templateIds,omitempty" jsonschema:"optional template ids to apply"`
	TemplateNames   []string `json:"templateNames,omitempty" jsonschema:"optional template names to apply"`
	OnExisting      string   `json:"onExisting,omitempty" jsonschema:"how to handle existing files: regenerate, skip_found, or error"`
	FocusQueue      bool     `json:"focusQueue,omitempty" jsonschema:"switch the UI to the queue tab after enqueueing"`
	SelectTemplates *bool    `json:"selectTemplates,omitempty" jsonschema:"persist the chosen templates in UI selection; defaults to true when omitted"`
}

type StartQueueArgs struct {
	WorkerID   string `json:"workerId,omitempty" jsonschema:"optional remote worker id"`
	WorkerName string `json:"workerName,omitempty" jsonschema:"optional remote worker display name"`
}

type UpdateTextControlArgs struct {
	TaskID string `json:"taskId" jsonschema:"queue task id awaiting text control"`
	Text   string `json:"text" jsonschema:"replacement text for the mini editor draft"`
}

type ConfirmTextControlArgs struct {
	TaskID string `json:"taskId" jsonschema:"queue task id awaiting text control"`
	Text   string `json:"text,omitempty" jsonschema:"optional final text to confirm; defaults to the current staged draft"`
}

type GetGalleryPreviewArgs struct {
	LimitPerTask             int      `json:"limitPerTask,omitempty" jsonschema:"maximum number of media items to return per task; defaults to 3"`
	LimitPerTemplate         int      `json:"limitPerTemplate,omitempty" jsonschema:"maximum number of media items to return per template before applying the task-wide limit"`
	TaskNames                []string `json:"taskNames,omitempty" jsonschema:"optional task names to include"`
	IncludePrompts           *bool    `json:"includePrompts,omitempty" jsonschema:"include the original generation prompt for each media item; defaults to true"`
	OnlyAwaitingImageControl *bool    `json:"onlyAwaitingImageControl,omitempty" jsonschema:"limit results to tasks currently waiting for image control; defaults to true"`
}

type NavigateArgs struct {
	Path string `json:"path" jsonschema:"target UI path such as text.translate, queue, or gallery"`
}

type GoogleMonitorGetItemsArgs struct {
	SheetID string `json:"sheetId" jsonschema:"id of the target google sheet tab"`
}

type GoogleMonitorCreateTaskArgs struct {
	SheetID  string `json:"sheetId" jsonschema:"id of the target google sheet tab"`
	RowIndex int    `json:"rowIndex" jsonschema:"index of the row to create a task from"`
}

func Start(invoke Invoker, logger *slog.Logger) (*Server, error) {
	implementation := &mcp.Implementation{
		Name:    "soloveyko-agent-control",
		Version: utils.AppVersion,
	}

	mcpServer := mcp.NewServer(implementation, &mcp.ServerOptions{
		Instructions: "Control the running Soloveyko.AI desktop app through high-level queue, draft, and review actions.",
		Logger:       logger,
	})

	addTools(mcpServer, invoke)

	handler := mcp.NewStreamableHTTPHandler(func(*http.Request) *mcp.Server {
		return mcpServer
	}, &mcp.StreamableHTTPOptions{
		Logger:         logger,
		SessionTimeout: 30 * time.Minute,
	})

	mux := http.NewServeMux()
	mux.Handle("/mcp", handler)
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]any{"ok": true})
	})

	address := resolveAddress()
	listener, err := net.Listen("tcp", address)
	if err != nil {
		return nil, fmt.Errorf("listen on %s: %w", address, err)
	}

	status := Status{
		Enabled:   true,
		Transport: "streamable_http",
		Address:   listener.Addr().String(),
		Endpoint:  fmt.Sprintf("http://%s/mcp", listener.Addr().String()),
		Server:    implementation.Name,
		Version:   implementation.Version,
	}

	httpServer := &http.Server{
		Handler:           mux,
		ReadHeaderTimeout: 5 * time.Second,
	}

	srv := &Server{
		httpServer: httpServer,
		listener:   listener,
		status:     status,
	}

	go func() {
		if err := httpServer.Serve(listener); err != nil && err != http.ErrServerClosed {
			if logger != nil {
				logger.Error("mcp http server stopped", "error", err)
			}
		}
	}()

	return srv, nil
}

func (s *Server) Close() error {
	if s == nil || s.httpServer == nil {
		return nil
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return s.httpServer.Shutdown(ctx)
}

func (s *Server) Status() Status {
	if s == nil {
		return Status{}
	}
	return s.status
}

func resolveAddress() string {
	if addr := strings.TrimSpace(os.Getenv("SOLOVEYKO_MCP_ADDR")); addr != "" {
		return addr
	}
	return "127.0.0.1:39245"
}

func addTools(server *mcp.Server, invoke Invoker) {
	mcp.AddTool(server, &mcp.Tool{Name: "set_main_text", Description: "Set the main draft text on the translate or rewrite tab."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args SetMainTextArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "set_main_text", args)
			if err != nil {
				return nil, nil, err
			}
			tab, _ := out["tab"].(string)
			return textResult(fmt.Sprintf("Updated %s draft text.", fallbackString(tab, "translate"))), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "get_main_text", Description: "Read the current main draft text from the translate or rewrite tab."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args GetMainTextArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "get_main_text", args)
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "select_templates", Description: "Select one or more pipeline templates for future enqueue actions."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args SelectTemplatesArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "select_templates", args)
			if err != nil {
				return nil, nil, err
			}
			names := stringListFromAny(out["selectedTemplateNames"])
			return textResult(fmt.Sprintf("Selected templates: %s.", strings.Join(names, ", "))), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "enqueue_task", Description: "Create one or more queue tasks from the current draft and optional template selection."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args EnqueueTaskArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "enqueue_task", args)
			if err != nil {
				return nil, nil, err
			}
			if queued, _ := out["queued"].(bool); !queued {
				return textResult("Existing files were detected and the task was not queued."), out, nil
			}
			return textResult("Task was added to the queue."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "start_queue", Description: "Start processing the current queue locally or on a selected remote worker."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args StartQueueArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "start_queue", args)
			if err != nil {
				return nil, nil, err
			}
			mode, _ := out["mode"].(string)
			return textResult(fmt.Sprintf("Queue started in %s mode.", fallbackString(mode, "local"))), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "continue_image_control", Description: "Continue processing after image control is ready in the gallery stage."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "continue_image_control", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return textResult("Image control batch resumed."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "get_pending_text_controls", Description: "List queue tasks that are waiting for text control review in the mini editor."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "get_pending_text_controls", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "update_text_control", Description: "Update the staged text inside a pending mini-editor review without confirming it yet."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args UpdateTextControlArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "update_text_control", args)
			if err != nil {
				return nil, nil, err
			}
			return textResult("Updated text control draft."), out, nil
		})

	// Google Monitor Tools
	mcp.AddTool(server, &mcp.Tool{Name: "google_monitor_scan", Description: "Trigger a fresh scan of all configured Google Sheets in the monitor."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "google_monitor_scan", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return textResult("Google Monitor scan completed."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "google_monitor_get_tabs", Description: "List all configured Google Sheet tabs in the monitor."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "google_monitor_get_tabs", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "google_monitor_get_items", Description: "Retrieve scanned items for a specific Google Sheet tab."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args GoogleMonitorGetItemsArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "google_monitor_get_items", args)
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "google_monitor_create_task", Description: "Create a queue task from a specific row in a Google Sheet monitor tab."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args GoogleMonitorCreateTaskArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "google_monitor_create_task", args)
			if err != nil {
				return nil, nil, err
			}
			return textResult("Task creation triggered from Google Monitor."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "confirm_text_control", Description: "Confirm a pending text-control review and continue the pipeline."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args ConfirmTextControlArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "confirm_text_control", args)
			if err != nil {
				return nil, nil, err
			}
			return textResult("Confirmed the pending text control."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "get_queue_state", Description: "Read the current queue state, task statuses, and completion signal."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "get_queue_state", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "get_gallery_preview", Description: "Read the first image or video items from the gallery for each task, including absolute file paths that the agent can show in chat."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args GetGalleryPreviewArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "get_gallery_preview", args)
			if err != nil {
				return nil, nil, err
			}
			return jsonTextResult(out), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "clear_queue", Description: "Clear all queued tasks and reset queue state."},
		func(ctx context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "clear_queue", map[string]any{})
			if err != nil {
				return nil, nil, err
			}
			return textResult("Queue cleared."), out, nil
		})

	mcp.AddTool(server, &mcp.Tool{Name: "navigate", Description: "Navigate the visible UI to a specific app path when needed."},
		func(ctx context.Context, _ *mcp.CallToolRequest, args NavigateArgs) (*mcp.CallToolResult, map[string]any, error) {
			out, err := invokeMap(ctx, invoke, "navigate", args)
			if err != nil {
				return nil, nil, err
			}
			return textResult(fmt.Sprintf("Navigated to %s.", args.Path)), out, nil
		})
}

func invokeMap(ctx context.Context, invoke Invoker, action string, params any) (map[string]any, error) {
	raw, err := invoke(ctx, action, params)
	if err != nil {
		return nil, err
	}
	if len(bytes.TrimSpace(raw)) == 0 {
		return map[string]any{}, nil
	}

	out := map[string]any{}
	if err := json.Unmarshal(raw, &out); err != nil {
		return nil, fmt.Errorf("decode %s response: %w", action, err)
	}
	return out, nil
}

func textResult(message string) *mcp.CallToolResult {
	return &mcp.CallToolResult{
		Content: []mcp.Content{
			&mcp.TextContent{Text: message},
		},
	}
}

func jsonTextResult(value any) *mcp.CallToolResult {
	payload, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		payload, _ = json.Marshal(value)
	}

	return &mcp.CallToolResult{
		Content: []mcp.Content{
			&mcp.TextContent{Text: string(payload)},
		},
	}
}

func fallbackString(value string, fallback string) string {
	if strings.TrimSpace(value) == "" {
		return fallback
	}
	return value
}

func stringListFromAny(value any) []string {
	rawList, ok := value.([]any)
	if !ok {
		return nil
	}
	result := make([]string, 0, len(rawList))
	for _, item := range rawList {
		if text, ok := item.(string); ok {
			result = append(result, text)
		}
	}
	return result
}

func sliceLength(value any) int {
	rawList, ok := value.([]any)
	if !ok {
		return 0
	}
	return len(rawList)
}
