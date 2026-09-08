use axum::{
    response::{IntoResponse, Response},
    http::{header, HeaderValue, StatusCode},
    Json,
};
use serde_json::json;

pub async fn scalar_docs_html() -> impl IntoResponse {
    let html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>VoxForg API Reference | Studio Neural Speech & DSP</title>
    <meta name="description" content="Interactive API documentation for VoxForg neural audio synthesis, graph pipelines, and automated QA." />
    <style>
      body {
        margin: 0;
        padding: 0;
        background-color: #0b0f19;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      }
    </style>
  </head>
  <body>
    <script
      id="api-reference"
      data-url="/openapi.json"
      data-configuration='{"theme":"purple","layout":"modern","darkMode":true,"showSidebar":true}'
    ></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))
        .body(axum::body::Body::from(html))
        .unwrap()
}

pub async fn openapi_spec() -> Json<serde_json::Value> {
    let spec = json!({
        "openapi": "3.1.0",
        "info": {
            "title": "VoxForg Neural Audio Engine API",
            "version": "0.1.0",
            "description": "Studio-grade neural speech synthesis, modular DSP audio graph pipelines, and automated A/B voice quality testing."
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Local VoxForg Server"
            }
        ],
        "tags": [
            { "name": "Speech", "description": "OpenAI-compatible speech synthesis" },
            { "name": "Pipeline", "description": "Modular DSP and audio graph processing" },
            { "name": "Quality Assurance", "description": "A/B voice testing and objective audio quality metrics" },
            { "name": "Models", "description": "Installed and supported neural TTS engines" },
            { "name": "Voices", "description": "Available voice profiles and acoustic properties" },
            { "name": "Health", "description": "System health and readiness diagnostics" }
        ],
        "paths": {
            "/health": {
                "get": {
                    "tags": ["Health"],
                    "summary": "Liveness probe",
                    "description": "Quick check to verify server process is alive.",
                    "responses": {
                        "200": {
                            "description": "Server is healthy",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "ok" },
                                            "version": { "type": "string", "example": "0.1.0" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/health/ready": {
                "get": {
                    "tags": ["Health"],
                    "summary": "Readiness probe",
                    "description": "Verifies registered engines, hardware acceleration status, and storage readiness.",
                    "responses": {
                        "200": {
                            "description": "Server is ready to accept synthesis traffic",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "ready" },
                                            "version": { "type": "string", "example": "0.1.0" },
                                            "registered_engines": {
                                                "type": "array",
                                                "items": { "type": "string" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/models": {
                "get": {
                    "tags": ["Models"],
                    "summary": "List TTS models",
                    "description": "Lists all available neural TTS models and engines.",
                    "security": [{ "BearerAuth": [] }],
                    "responses": {
                        "200": {
                            "description": "List of available models",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ModelListResponse" }
                                }
                            }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" }
                    }
                }
            },
            "/v1/voices": {
                "get": {
                    "tags": ["Voices"],
                    "summary": "List voice profiles",
                    "description": "Retrieves available voices with optional filtering by language code and engine.",
                    "security": [{ "BearerAuth": [] }],
                    "parameters": [
                        {
                            "name": "language",
                            "in": "query",
                            "description": "Filter by language code (e.g., 'en-US')",
                            "required": false,
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "engine",
                            "in": "query",
                            "description": "Filter by TTS engine ID",
                            "required": false,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Voice catalog",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/VoicesResponse" }
                                }
                            }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" }
                    }
                }
            },
            "/v1/audio/speech": {
                "post": {
                    "tags": ["Speech"],
                    "summary": "Synthesize speech",
                    "description": "Generate high-fidelity audio from text using neural synthesis models (OpenAI-compatible).",
                    "security": [{ "BearerAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/OpenAiSpeechRequest" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Generated audio stream (RIFF WAV format)",
                            "content": {
                                "audio/wav": {
                                    "schema": {
                                        "type": "string",
                                        "format": "binary"
                                    }
                                }
                            }
                        },
                        "400": { "$ref": "#/components/responses/BadRequest" },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "404": { "$ref": "#/components/responses/NotFound" },
                        "500": { "$ref": "#/components/responses/ServerError" }
                    }
                }
            },
            "/v1/pipeline/execute": {
                "post": {
                    "tags": ["Pipeline"],
                    "summary": "Execute audio graph pipeline",
                    "description": "Executes a directed acyclic audio processing graph supporting synthesis, DSP filtering (silence trimming, 3-band parametric EQ, envelope dynamic compression, brickwall limiting), normalization, and multi-track audio merging.",
                    "security": [{ "BearerAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/PipelineExecuteRequest" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Mastered audio buffer (RIFF WAV format)",
                            "content": {
                                "audio/wav": {
                                    "schema": {
                                        "type": "string",
                                        "format": "binary"
                                    }
                                }
                            }
                        },
                        "400": { "$ref": "#/components/responses/BadRequest" },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "500": { "$ref": "#/components/responses/ServerError" }
                    }
                }
            },
            "/v1/qa/ab-test": {
                "post": {
                    "tags": ["Quality Assurance"],
                    "summary": "Run A/B voice model evaluation",
                    "description": "Synthesizes identical text across two candidate configurations and computes latency, SNR, PESQ proxy score, clipping ratios, and duration metrics with automated recommendation.",
                    "security": [{ "BearerAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/AbTestScenario" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "A/B comparison results and metrics",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/AbTestComparison" }
                                }
                            }
                        },
                        "400": { "$ref": "#/components/responses/BadRequest" },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "500": { "$ref": "#/components/responses/ServerError" }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "API Key",
                    "description": "API key passed as Bearer token in the Authorization header"
                }
            },
            "responses": {
                "BadRequest": {
                    "description": "Invalid parameter or payload (RFC 7807 Problem Details)",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ProblemDetails" }
                        }
                    }
                },
                "Unauthorized": {
                    "description": "Authentication token missing or invalid",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ProblemDetails" }
                        }
                    }
                },
                "NotFound": {
                    "description": "Requested resource not found",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ProblemDetails" }
                        }
                    }
                },
                "ServerError": {
                    "description": "Internal server execution failure",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ProblemDetails" }
                        }
                    }
                }
            },
            "schemas": {
                "OpenAiSpeechRequest": {
                    "type": "object",
                    "required": ["model", "input", "voice"],
                    "properties": {
                        "model": { "type": "string", "example": "kokoro" },
                        "input": { "type": "string", "example": "Welcome to VoxForg studio neural voice synthesis." },
                        "voice": { "type": "string", "example": "af_bella" },
                        "response_format": {
                            "type": "string",
                            "enum": ["wav", "mp3", "ogg", "flac"],
                            "default": "wav"
                        },
                        "speed": { "type": "number", "minimum": 0.25, "maximum": 4.0, "default": 1.0 },
                        "pitch": { "type": "number", "minimum": -50.0, "maximum": 50.0, "default": 0.0 }
                    }
                },
                "PipelineExecuteRequest": {
                    "type": "object",
                    "required": ["pipeline"],
                    "properties": {
                        "pipeline": { "$ref": "#/components/schemas/PipelineDefinition" },
                        "input_text": { "type": "string", "nullable": true }
                    }
                },
                "PipelineDefinition": {
                    "type": "object",
                    "required": ["id", "name", "nodes", "edges"],
                    "properties": {
                        "id": { "type": "string", "format": "uuid" },
                        "name": { "type": "string", "example": "Broadcast Voice Mastering" },
                        "description": { "type": "string", "nullable": true },
                        "nodes": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/PipelineNode" }
                        },
                        "edges": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/PipelineEdge" }
                        }
                    }
                },
                "PipelineNode": {
                    "type": "object",
                    "required": ["id", "name", "node_type", "params"],
                    "properties": {
                        "id": { "type": "string", "example": "node_filter_1" },
                        "name": { "type": "string", "example": "Studio DSP Mastering" },
                        "node_type": {
                            "type": "string",
                            "enum": ["text_input", "synthesizer", "audio_filter", "audio_merger", "output_sink"]
                        },
                        "params": {
                            "type": "object",
                            "description": "Node parameters. For audio_filter: silence_trim, eq, compressor, limiter, normalize_peak, gain_db."
                        }
                    }
                },
                "PipelineEdge": {
                    "type": "object",
                    "required": ["id", "from_node", "to_node"],
                    "properties": {
                        "id": { "type": "string", "example": "edge_1" },
                        "from_node": { "type": "string", "example": "node_synth_1" },
                        "to_node": { "type": "string", "example": "node_filter_1" }
                    }
                },
                "AbTestScenario": {
                    "type": "object",
                    "required": ["name", "text", "variant_a", "variant_b"],
                    "properties": {
                        "name": { "type": "string", "example": "Bella vs Nicole Model Test" },
                        "text": { "type": "string", "example": "High quality neural speech sample for comparison." },
                        "variant_a": {
                            "type": "object",
                            "properties": {
                                "voice_id": { "type": "string" },
                                "speed": { "type": "number" },
                                "pitch": { "type": "number" }
                            }
                        },
                        "variant_b": {
                            "type": "object",
                            "properties": {
                                "voice_id": { "type": "string" },
                                "speed": { "type": "number" },
                                "pitch": { "type": "number" }
                            }
                        }
                    }
                },
                "AbTestComparison": {
                    "type": "object",
                    "properties": {
                        "scenario_name": { "type": "string" },
                        "variant_a": { "$ref": "#/components/schemas/VariantResult" },
                        "variant_b": { "$ref": "#/components/schemas/VariantResult" },
                        "recommended_variant": { "type": "string", "example": "Variant A" },
                        "recommendation_rationale": { "type": "string" }
                    }
                },
                "VariantResult": {
                    "type": "object",
                    "properties": {
                        "variant_name": { "type": "string" },
                        "latency_ms": { "type": "number" },
                        "rtf": { "type": "number" },
                        "audio_metrics": { "$ref": "#/components/schemas/AudioMetrics" }
                    }
                },
                "AudioMetrics": {
                    "type": "object",
                    "properties": {
                        "duration_seconds": { "type": "number" },
                        "sample_rate": { "type": "integer" },
                        "peak_amplitude": { "type": "number" },
                        "rms_db": { "type": "number" },
                        "estimated_snr_db": { "type": "number" },
                        "clipping_ratio": { "type": "number" },
                        "pesq_proxy_score": { "type": "number" }
                    }
                },
                "ModelListResponse": {
                    "type": "object",
                    "properties": {
                        "object": { "type": "string", "example": "list" },
                        "data": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": { "type": "string" },
                                    "object": { "type": "string" },
                                    "created": { "type": "integer" },
                                    "owned_by": { "type": "string" }
                                }
                            }
                        }
                    }
                },
                "VoicesResponse": {
                    "type": "object",
                    "properties": {
                        "voices": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/Voice" }
                        },
                        "total": { "type": "integer" }
                    }
                },
                "Voice": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "name": { "type": "string" },
                        "engine_id": { "type": "string" },
                        "language": { "type": "string" },
                        "gender": { "type": "string" },
                        "sample_rate_hz": { "type": "integer" },
                        "tags": { "type": "array", "items": { "type": "string" } },
                        "description": { "type": "string", "nullable": true }
                    }
                },
                "ProblemDetails": {
                    "type": "object",
                    "required": ["type", "title", "status", "detail", "instance"],
                    "properties": {
                        "type": { "type": "string", "example": "https://voxforg.org/errors/invalid-parameter" },
                        "title": { "type": "string", "example": "Invalid Speed Parameter" },
                        "status": { "type": "integer", "example": 400 },
                        "detail": { "type": "string", "example": "The 'speed' parameter must be a finite number between 0.25 and 4.0" },
                        "instance": { "type": "string", "example": "/v1/audio/speech" }
                    }
                }
            }
        }
    });

    Json(spec)
}
