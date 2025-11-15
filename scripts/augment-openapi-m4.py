#!/usr/bin/env python3
"""
Augment OpenAPI spec with M4 (Graph/Sched/LLM/Logs) endpoints.
"""

import json

def main():
    # Read existing spec
    with open('openapi.json', 'r') as f:
        spec = json.load(f)

    # Scheduling paths
    sched_paths = {
        "/api/v1/sched/workloads": {
            "get": {
                "tags": ["scheduling"],
                "summary": "Get list of workloads",
                "operationId": "schedWorkloads",
                "responses": {
                    "200": {"description": "List of workloads", "content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/Workload"}}}}}
                }
            }
        },
        "/api/v1/sched/priorities": {
            "post": {
                "tags": ["scheduling"],
                "summary": "Set workload priority",
                "operationId": "schedSetPriority",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SetPriorityRequest"}}}},
                "responses": {
                    "200": {"description": "Priority updated", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SchedResponse"}}}}
                }
            }
        },
        "/api/v1/sched/affinity": {
            "post": {
                "tags": ["scheduling"],
                "summary": "Set workload CPU affinity",
                "operationId": "schedSetAffinity",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SetAffinityRequest"}}}},
                "responses": {
                    "200": {"description": "Affinity updated", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SchedResponse"}}}}
                }
            }
        },
        "/api/v1/sched/feature": {
            "post": {
                "tags": ["scheduling"],
                "summary": "Toggle scheduling feature",
                "operationId": "schedSetFeature",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SetFeatureRequest"}}}},
                "responses": {
                    "200": {"description": "Feature toggled", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SchedResponse"}}}}
                }
            }
        },
        "/api/v1/sched/circuit-breaker": {
            "get": {
                "tags": ["scheduling"],
                "summary": "Get circuit breaker state",
                "operationId": "schedCircuitBreakerStatus",
                "responses": {
                    "200": {"description": "Circuit breaker state", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/CircuitBreakerState"}}}}
                }
            }
        },
        "/api/v1/sched/circuit-breaker/reset": {
            "post": {
                "tags": ["scheduling"],
                "summary": "Reset circuit breaker",
                "operationId": "schedCircuitBreakerReset",
                "responses": {
                    "200": {"description": "Circuit breaker reset", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SchedResponse"}}}}
                }
            }
        }
    }

    # LLM paths
    llm_paths = {
        "/api/v1/llm/load": {
            "post": {
                "tags": ["llm"],
                "summary": "Load LLM model",
                "operationId": "llmLoad",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/LoadModelRequest"}}}},
                "responses": {
                    "200": {"description": "Model loaded", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/LoadModelResponse"}}}}
                }
            }
        },
        "/api/v1/llm/infer": {
            "post": {
                "tags": ["llm"],
                "summary": "Submit inference request",
                "operationId": "llmInfer",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/InferRequest"}}}},
                "responses": {
                    "200": {"description": "Inference started", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/InferResponse"}}}}
                }
            }
        },
        "/api/v1/llm/audit": {
            "get": {
                "tags": ["llm"],
                "summary": "Get inference audit log",
                "operationId": "llmAudit",
                "responses": {
                    "200": {"description": "Audit log entries", "content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/AuditEntry"}}}}}
                }
            }
        },
        "/api/v1/llm/status": {
            "get": {
                "tags": ["llm"],
                "summary": "Get LLM status",
                "operationId": "llmStatus",
                "responses": {
                    "200": {"description": "LLM status", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/LlmStatus"}}}}
                }
            }
        }
    }

    # Logs paths
    logs_paths = {
        "/api/v1/logs/tail": {
            "get": {
                "tags": ["logs"],
                "summary": "Tail logs with optional filters",
                "operationId": "logsTail",
                "parameters": [
                    {"name": "limit", "in": "query", "schema": {"type": "integer"}, "required": False},
                    {"name": "level", "in": "query", "schema": {"type": "string"}, "required": False},
                    {"name": "source", "in": "query", "schema": {"type": "string"}, "required": False}
                ],
                "responses": {
                    "200": {"description": "Log entries", "content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/LogEntry"}}}}}
                }
            }
        },
        "/api/v1/runs/start": {
            "post": {
                "tags": ["runs"],
                "summary": "Start a new run",
                "operationId": "runsStart",
                "requestBody": {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/StartRunRequest"}}}},
                "responses": {
                    "200": {"description": "Run started", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/StartRunResponse"}}}}
                }
            }
        },
        "/api/v1/runs/stop": {
            "post": {
                "tags": ["runs"],
                "summary": "Stop current run",
                "operationId": "runsStop",
                "responses": {
                    "200": {"description": "Run stopped", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/StopRunResponse"}}}}
                }
            }
        },
        "/api/v1/runs/list": {
            "get": {
                "tags": ["runs"],
                "summary": "List run history",
                "operationId": "runsList",
                "responses": {
                    "200": {"description": "Run history", "content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/RunHistoryEntry"}}}}}
                }
            }
        },
        "/api/v1/runs/{runId}/export": {
            "get": {
                "tags": ["runs"],
                "summary": "Export run logs and metrics",
                "operationId": "runsExport",
                "parameters": [{"name": "runId", "in": "path", "required": True, "schema": {"type": "string"}}],
                "responses": {
                    "200": {"description": "Run snapshot exported", "content": {"application/json": {}}}
                }
            }
        }
    }

    # Merge all paths
    spec['paths'].update(sched_paths)
    spec['paths'].update(llm_paths)
    spec['paths'].update(logs_paths)

    # M4 Schemas - Graph
    graph_schemas = {
        "CreateGraphResponse": {"type": "object", "properties": {"graphId": {"type": "string"}}, "required": ["graphId"]},
        "AddChannelRequest": {"type": "object", "properties": {"graphId": {"type": "string"}, "cap": {"type": "integer"}}, "required": ["graphId", "cap"]},
        "AddChannelResponse": {"type": "object", "properties": {"channelId": {"type": "string"}}, "required": ["channelId"]},
        "AddOperatorRequest": {
            "type": "object",
            "properties": {
                "graphId": {"type": "string"},
                "opId": {"type": "string"},
                "in": {"type": "string"},
                "out": {"type": "string"},
                "prio": {"type": "integer"},
                "stage": {"type": "string"},
                "inSchema": {"type": "string"},
                "outSchema": {"type": "string"}
            },
            "required": ["graphId", "opId"]
        },
        "AddOperatorResponse": {"type": "object", "properties": {"operatorId": {"type": "string"}}, "required": ["operatorId"]},
        "StartGraphRequest": {"type": "object", "properties": {"graphId": {"type": "string"}, "steps": {"type": "integer"}}, "required": ["graphId", "steps"]},
        "StartGraphResponse": {"type": "object", "properties": {"started": {"type": "boolean"}}, "required": ["started"]},
        "PredictRequest": {
            "type": "object",
            "properties": {"opId": {"type": "string"}, "latency_us": {"type": "integer"}, "depth": {"type": "integer"}, "prio": {"type": "integer"}},
            "required": ["opId", "latency_us", "depth"]
        },
        "PredictResponse": {"type": "object", "properties": {"predicted": {"type": "number"}, "conf": {"type": "number"}}},
        "FeedbackRequest": {"type": "object", "properties": {"opId": {"type": "string"}, "verdict": {"type": "string"}}, "required": ["opId", "verdict"]},
        "FeedbackResponse": {"type": "object", "properties": {"recorded": {"type": "boolean"}}, "required": ["recorded"]},
        "GraphOperator": {"type": "object", "properties": {"id": {"type": "string"}, "stage": {"type": "string"}, "prio": {"type": "integer"}, "state": {"type": "string"}}},
        "GraphChannel": {"type": "object", "properties": {"id": {"type": "string"}, "cap": {"type": "integer"}, "depth": {"type": "integer"}}},
        "GraphStats": {"type": "object", "properties": {"operator_count": {"type": "integer"}, "channel_count": {"type": "integer"}, "total_executions": {"type": "integer"}}},
        "GraphState": {
            "type": "object",
            "properties": {
                "operators": {"type": "array", "items": {"$ref": "#/components/schemas/GraphOperator"}},
                "channels": {"type": "array", "items": {"$ref": "#/components/schemas/GraphChannel"}},
                "stats": {"$ref": "#/components/schemas/GraphStats"}
            }
        },
        "ExportGraphRequest": {"type": "object", "properties": {"graphId": {"type": "string"}, "format": {"type": "string"}}, "required": ["graphId", "format"]},
        "ExportGraphResponse": {"type": "object", "properties": {"json": {"type": "string"}}, "required": ["json"]},
    }

    # M4 Schemas - Scheduling
    sched_schemas = {
        "Workload": {"type": "object", "properties": {"pid": {"type": "integer"}, "name": {"type": "string"}, "prio": {"type": "integer"}, "cpu": {"type": "integer"}, "state": {"type": "string"}}},
        "SetPriorityRequest": {"type": "object", "properties": {"pid": {"type": "integer"}, "prio": {"type": "integer"}}, "required": ["pid", "prio"]},
        "SetAffinityRequest": {"type": "object", "properties": {"pid": {"type": "integer"}, "cpuMask": {"type": "string"}}, "required": ["pid", "cpuMask"]},
        "SetFeatureRequest": {"type": "object", "properties": {"name": {"type": "string"}, "enable": {"type": "boolean"}}, "required": ["name", "enable"]},
        "SchedResponse": {"type": "object", "properties": {"ok": {"type": "boolean"}}, "required": ["ok"]},
        "CircuitBreakerState": {
            "type": "object",
            "properties": {
                "state": {"type": "string"},
                "consecutive_failures": {"type": "integer"},
                "failure_threshold": {"type": "integer"},
                "reset_timeout_us": {"type": "integer"}
            }
        },
    }

    # M4 Schemas - LLM
    llm_schemas = {
        "LoadModelRequest": {
            "type": "object",
            "properties": {
                "modelId": {"type": "string"},
                "wcetCycles": {"type": "integer"},
                "ctx": {"type": "integer"},
                "vocab": {"type": "integer"},
                "quant": {"type": "string"},
                "hash": {"type": "string"},
                "sig": {"type": "string"}
            },
            "required": ["modelId"]
        },
        "LoadModelResponse": {"type": "object", "properties": {"loaded": {"type": "boolean"}}, "required": ["loaded"]},
        "InferRequest": {"type": "object", "properties": {"text": {"type": "string"}, "maxTokens": {"type": "integer"}}, "required": ["text"]},
        "InferResponse": {"type": "object", "properties": {"requestId": {"type": "string"}}, "required": ["requestId"]},
        "AuditEntry": {
            "type": "object",
            "properties": {"id": {"type": "string"}, "modelId": {"type": "string"}, "tokens": {"type": "integer"}, "done": {"type": "boolean"}, "ts": {"type": "integer"}}
        },
        "LlmStatus": {
            "type": "object",
            "properties": {
                "budget": {"type": "integer"},
                "wcetCycles": {"type": "integer"},
                "periodNs": {"type": "integer"},
                "maxTokensPerPeriod": {"type": "integer"},
                "queueDepth": {"type": "integer"},
                "lastInferUs": {"type": "integer"}
            }
        },
    }

    # M4 Schemas - Logs
    logs_schemas = {
        "LogEntry": {"type": "object", "properties": {"ts": {"type": "integer"}, "level": {"type": "string"}, "source": {"type": "string"}, "msg": {"type": "string"}}},
        "RunProfile": {"type": "object", "properties": {"features": {"type": "array", "items": {"type": "string"}}, "bringup": {"type": "boolean"}}},
        "StartRunRequest": {"type": "object", "properties": {"profile": {"$ref": "#/components/schemas/RunProfile"}, "note": {"type": "string"}}, "required": ["profile"]},
        "StartRunResponse": {"type": "object", "properties": {"runId": {"type": "string"}}, "required": ["runId"]},
        "StopRunResponse": {"type": "object", "properties": {"ok": {"type": "boolean"}}, "required": ["ok"]},
        "RunHistoryEntry": {
            "type": "object",
            "properties": {
                "runId": {"type": "string"},
                "profile": {"$ref": "#/components/schemas/RunProfile"},
                "startedAt": {"type": "integer"},
                "stoppedAt": {"type": "integer"},
                "markers": {"type": "array", "items": {"type": "string"}}
            }
        },
    }

    # Merge all schemas
    spec['components']['schemas'].update(graph_schemas)
    spec['components']['schemas'].update(sched_schemas)
    spec['components']['schemas'].update(llm_schemas)
    spec['components']['schemas'].update(logs_schemas)

    # Write updated spec
    with open('openapi.json', 'w') as f:
        json.dump(spec, f, indent=2)

    print("✓ OpenAPI spec augmented with M4 endpoints")
    print(f"  Added {len(sched_paths) + len(llm_paths) + len(logs_paths)} paths (sched+llm+logs)")
    print(f"  Added {len(graph_schemas) + len(sched_schemas) + len(llm_schemas) + len(logs_schemas)} schemas")
    print(f"  Total paths: {len(spec['paths'])}")
    print(f"  Total schemas: {len(spec['components']['schemas'])}")

if __name__ == '__main__':
    main()
