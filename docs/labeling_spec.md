# Misinformation Labeling Specification

## Overview
This document defines the method for labeling posts as misinformation (demagoguery, fake news, etc.) within the XQ and ActivityPub protocols for the Bubbles ecosystem.

The goal is to allow trusted entities (or the community) to attach a "Label" to a specific post/note, indicating its reliability status.

## 1. XQ Protocol (Protobuf)

In XQ, we define a specialized `Label` message. This message is signed by the labeler and references the target message hash.

### Proto Definition

Add the following to `bubbles_xq.proto`:

```protobuf
syntax = "proto3";

package bubbles.xq;

// Enum for the type of label
enum LabelType {
    LABEL_TYPE_UNSPECIFIED = 0;
    LABEL_TYPE_MISINFORMATION = 1; // False information
    LABEL_TYPE_DISINFORMATION = 2; // Intentionally false
    LABEL_TYPE_SATIRE = 3;         // Satire/Parody
    LABEL_TYPE_MANIPULATED = 4;    // Manipulated media
    LABEL_TYPE_CONTEXT_ADDED = 5;  // Missing context
}

// The Label message
message Label {
    string id = 1;                 // Unique ID of this label
    string target_id = 2;          // ID (Hash) of the target message
    LabelType type = 3;            // Type of label
    string summary = 4;            // Short explanation (e.g. "This has been debunked")
    string evidence_url = 5;       // URL to fact check or evidence
    int64 created_at = 6;          // Timestamp
    string author_pubkey = 7;      // Public key of the labeler
    bytes signature = 8;           // Signature of the label
}

// Wrapper for streaming/transport
message XQMessage {
    oneof content {
        Note note = 1;
        Label label = 2; // Added Label support
    }
}
```

## 2. ActivityPub (JSON-LD)

For ActivityPub, we utilize the `Flag` activity or schema.org `Review` objects to express fact checks.
A common pattern involves an `Offer` of a `Review`, or a specific `Note` with `reviewAspect`.

We recommend using a `Note` (for compatibility) with an extension property for structured claims, or a `Flag` for moderation. For public labeling/community notes, a `Note` replying to the original with specific context is best.

### JSON Structure

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://bubbles.network/ns/v1"
  ],
  "id": "https://example.com/check/123",
  "type": "Note", // Or "Article" / "Review"
  "inReplyTo": "https://remote.server/posts/original-post-id",
  "content": "This post contains misleading information about...",
  "summary": "Misinformation Alert",
  "bubbles:label": {
    "type": "Misinformation",
    "verdict": "False",
    "evidence": "https://factcheck.org/example"
  },
  "attributedTo": "https://example.com/users/factchecker",
  "published": "2025-12-13T12:00:00Z"
}
```

## 3. Labeling Logic

### Verification
- **Authority**: Clients should only display labels from entities in the user's "Web of Trust" or a configured "Trusted Fact Checkers" list.
- **Signature**: All XQ Labels must be cryptographically verified against the `author_pubkey`.

### Display
- **Blurring**: Content marked as `MISINFORMATION` or `MANIPULATED` with high confidence from a trusted source may be blurred or hidden behind a click-through warning.
- **Context**: `CONTEXT_ADDED` labels should be displayed prominently below the post (similar to Community Notes).

## 4. AI-Automated Labeling (OpenRouter Integration)

To scale the verification process, we define an automated bot architecture leveraging Small Language Models (SLMs) via OpenRouter.

### Architecture
1. **Listener**: A bot listens to the public timeline (firehose) or specific reported posts.
2. **Analyzer**: The bot sends the post content to OpenRouter API (OpenAI-compatible).
3. **Decider**: The SLM (e.g., Llama 3 8B, Mistral 7B) analyzes the text for factual claims and potential misinformation.
4. **Action**: If a threshold is met, the bot signs and broadcasts a `Label` message.

### Recommended Models (SLMs)
Using OpenRouter allows access to cost-effective, high-performance lightweight models:
- `meta-llama/llama-3-8b-instruct`: Fast, good reasoning.
- `mistralai/mistral-7b-instruct`: Very efficient for short context.
- `google/gemma-7b-it`: Good balanced performance.

### Prompt Strategy
The system prompt should enforce output in a structured JSON format to easily map to the `Label` protocol.

**System Prompt Example:**
```text
You are an automated fact-checking assistant for a social network.
Analyze the following user post for potential misinformation, disinformation, or harmful content.
Focus on objective verifiability. usage of strong emotional language, and lack of sources.

Return your analysis in the following JSON format ONLY:
{
  "is_misinformation": boolean,
  "confidence_score": float (0.0-1.0),
  "label_type": "MISINFORMATION" | "DISINFORMATION" | "SATIRE" | "NONE",
  "reasoning_summary": "Short explanation (max 100 chars)",
  "suggested_evidence_keywords": ["keyword1", "keyword2"]
}
```

### Integration Logic
If `confidence_score` > 0.85 and `is_misinformation` is true, the bot will automatically issue a Label.
For scores between 0.6 and 0.85, it might flag for human review (internal queue).


## 5. Human-in-the-Loop Strategy

While AI automation provides scale, human judgment is essential for nuance and fail-safe operations.

### Hybrid Workflow
- **Jargon & Slang**: If the post contains a high density of terms specific to a sub-community (e.g., crypto slang, gaming jargon) that the SLM fails to parse confidently (low confidence score despite potentially dangerous keywords), it should be routed to a human moderator familiar with that context.
- **Vote/Report Escalation**: If trusted users explicitly report a labeled post as "Incorrect Label", it triggers a human review.

### Fallback Mechanism
1. **Rate Limits**: If the OpenRouter API hits a Rate Limit (429), the system must fallback to:
    - Queueing the post for later analysis (with backoff).
    - If the queue is full, flagging it for manual human review.
2. **Context Blindness**: If the AI cannot determine the truthfulness due to lack of external context (e.g., breaking news from seconds ago), it should output `confidence_score: 0.0` or a specific status `NEEDS_HUMAN_REVIEW`.
