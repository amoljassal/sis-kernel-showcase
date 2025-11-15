#!/usr/bin/env python3
"""
Augment OpenAPI spec with M3 (Autonomy + Memory) endpoints.

This script adds the missing autonomy and memory endpoints/schemas
to the existing openapi.json without requiring Rust compilation.
"""

import json
import sys

def main():
    # Read existing spec
    with open('openapi.json', 'r') as f:
        spec = json.load(f)

    # Add new paths
    new_paths = {
        "/api/v1/autonomy/on": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Enable autonomy",
                "operationId": "autonomyOn",
                "responses": {
                    "200": {
                        "description": "Autonomy enabled",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/off": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Disable autonomy",
                "operationId": "autonomyOff",
                "responses": {
                    "200": {
                        "description": "Autonomy disabled",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/reset": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Reset autonomy state",
                "operationId": "autonomyReset",
                "responses": {
                    "200": {
                        "description": "Autonomy reset",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/interval": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Set autonomy interval",
                "operationId": "autonomySetInterval",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "interval_ms": {"type": "integer"}
                                },
                                "required": ["interval_ms"]
                            }
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Interval updated",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/conf-threshold": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Set confidence threshold",
                "operationId": "autonomySetThreshold",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "threshold": {"type": "integer"}
                                },
                                "required": ["threshold"]
                            }
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Threshold updated",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/status": {
            "get": {
                "tags": ["autonomy"],
                "summary": "Get autonomy status",
                "operationId": "autonomyStatus",
                "responses": {
                    "200": {
                        "description": "Autonomy status",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AutonomyStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/audit": {
            "get": {
                "tags": ["autonomy"],
                "summary": "Get autonomy audit log",
                "operationId": "autonomyAudit",
                "parameters": [
                    {
                        "name": "last",
                        "in": "query",
                        "schema": {"type": "integer"},
                        "required": False
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Audit entries",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "array",
                                    "items": {"$ref": "#/components/schemas/AutonomyDecision"}
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/explain": {
            "get": {
                "tags": ["autonomy"],
                "summary": "Explain a decision",
                "operationId": "autonomyExplain",
                "parameters": [
                    {
                        "name": "id",
                        "in": "query",
                        "schema": {"type": "string"},
                        "required": True
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Decision explanation",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/ExplainResponse"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/preview": {
            "post": {
                "tags": ["autonomy"],
                "summary": "Preview next decisions",
                "operationId": "autonomyPreview",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/PreviewRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Preview results",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/PreviewResponse"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/autonomy/whatif": {
            "post": {
                "tags": ["autonomy"],
                "summary": "What-if scenario analysis",
                "operationId": "autonomyWhatIf",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/WhatIfRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "What-if results",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/WhatIfResponse"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/mem/approvals": {
            "get": {
                "tags": ["memory"],
                "summary": "Get pending approvals",
                "operationId": "memGetApprovals",
                "parameters": [
                    {
                        "name": "limit",
                        "in": "query",
                        "schema": {"type": "integer"},
                        "required": False
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Pending operations",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "array",
                                    "items": {"$ref": "#/components/schemas/PendingOperation"}
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/mem/approval": {
            "post": {
                "tags": ["memory"],
                "summary": "Toggle approval mode",
                "operationId": "memApprovalToggle",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/ApprovalToggleRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Approval mode updated",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/MemoryApprovalStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/mem/approve": {
            "post": {
                "tags": ["memory"],
                "summary": "Approve N operations",
                "operationId": "memApprove",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/ApproveRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Operations approved",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/MemoryApprovalStatus"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/mem/reject": {
            "post": {
                "tags": ["memory"],
                "summary": "Reject operation(s)",
                "operationId": "memReject",
                "requestBody": {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/RejectRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Operations rejected",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/MemoryApprovalStatus"}
                            }
                        }
                    }
                }
            }
        }
    }

    # Add new schemas
    new_schemas = {
        "AutonomyStatus": {
            "type": "object",
            "properties": {
                "enabled": {"type": "boolean"},
                "safe_mode": {"type": "boolean"},
                "learning_frozen": {"type": "boolean"},
                "interval_ms": {"type": "integer"},
                "threshold": {"type": "integer"},
                "total_decisions": {"type": "integer"},
                "accepted": {"type": "integer"},
                "deferred": {"type": "integer"},
                "watchdog_low_reward": {"type": "integer"},
                "watchdog_high_td_error": {"type": "integer"}
            },
            "required": ["enabled", "interval_ms", "threshold", "total_decisions"]
        },
        "AutonomyDecision": {
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "timestamp_us": {"type": "integer"},
                "action": {"type": "string"},
                "confidence": {"type": "integer"},
                "reward": {"type": "number", "format": "float"},
                "executed": {"type": "boolean"}
            },
            "required": ["id", "timestamp_us", "action", "confidence"]
        },
        "AttentionWeight": {
            "type": "object",
            "properties": {
                "feature": {"type": "string"},
                "weight": {"type": "number", "format": "float"}
            },
            "required": ["feature", "weight"]
        },
        "ExplainResponse": {
            "type": "object",
            "properties": {
                "decision_id": {"type": "string"},
                "explanation": {"type": "string"},
                "reasoning": {"type": "string"},
                "attention": {
                    "type": "array",
                    "items": {"$ref": "#/components/schemas/AttentionWeight"}
                },
                "context": {"type": "object"}
            },
            "required": ["decision_id", "explanation"]
        },
        "PreviewRequest": {
            "type": "object",
            "properties": {
                "count": {"type": "integer"}
            },
            "required": ["count"]
        },
        "PreviewResponse": {
            "type": "object",
            "properties": {
                "decisions": {
                    "type": "array",
                    "items": {"$ref": "#/components/schemas/AutonomyDecision"}
                },
                "would_execute": {"type": "boolean"},
                "confidence": {"type": "integer"}
            },
            "required": ["decisions"]
        },
        "WhatIfRequest": {
            "type": "object",
            "properties": {
                "overrides": {"type": "object"}
            },
            "required": ["overrides"]
        },
        "WhatIfResponse": {
            "type": "object",
            "properties": {
                "scenario": {"$ref": "#/components/schemas/PreviewResponse"},
                "warnings": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["scenario"]
        },
        "MemoryApprovalStatus": {
            "type": "object",
            "properties": {
                "enabled": {"type": "boolean"},
                "pending_count": {"type": "integer"},
                "total_approved": {"type": "integer"},
                "total_rejected": {"type": "integer"}
            },
            "required": ["enabled", "pending_count"]
        },
        "PendingOperation": {
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "operation_type": {"type": "string"},
                "confidence": {"type": "integer"},
                "risk_score": {"type": "string"},
                "reason": {"type": "string"},
                "timestamp_us": {"type": "integer"}
            },
            "required": ["id", "operation_type"]
        },
        "ApprovalToggleRequest": {
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["on", "off", "status"]}
            },
            "required": ["action"]
        },
        "ApproveRequest": {
            "type": "object",
            "properties": {
                "n": {"type": "integer"}
            },
            "required": ["n"]
        },
        "RejectRequest": {
            "type": "object",
            "properties": {
                "id": {"type": "string"}
            }
        }
    }

    # Merge paths
    spec['paths'].update(new_paths)

    # Merge schemas
    spec['components']['schemas'].update(new_schemas)

    # Write updated spec
    with open('openapi.json', 'w') as f:
        json.dump(spec, f, indent=2)

    print("✓ OpenAPI spec augmented with M3 endpoints")
    print(f"  Added {len(new_paths)} paths")
    print(f"  Added {len(new_schemas)} schemas")

if __name__ == '__main__':
    main()
