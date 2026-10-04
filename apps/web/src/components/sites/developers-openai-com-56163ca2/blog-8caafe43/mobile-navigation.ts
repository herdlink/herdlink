export interface MobileNavigationEntry { kind: string; text: string; href: string | null; active: boolean; classes: string; }
export interface MobileNavigationPanel { id: string; label: string; defaultVariant: string; options: { label: string; id: string; href: string; isHome: string }[]; variants: { id: string; entries: MobileNavigationEntry[] }[]; }

export const mobileNavigationPanels: MobileNavigationPanel[] = [
  {
    "id": "mobile-nav-tab-1",
    "defaultVariant": "mobile-nav-tab-1-variant-0",
    "options": [
      {
        "label": "Overview",
        "id": "mobile-nav-tab-1-variant-0",
        "href": "/api/docs",
        "isHome": "true"
      },
      {
        "label": "Models",
        "id": "mobile-nav-tab-1-variant-1",
        "href": "/api/docs/models",
        "isHome": "false"
      },
      {
        "label": "Agents",
        "id": "mobile-nav-tab-1-variant-2",
        "href": "/api/docs/guides/agents",
        "isHome": "false"
      },
      {
        "label": "Tools",
        "id": "mobile-nav-tab-1-variant-3",
        "href": "/api/docs/guides/tools",
        "isHome": "false"
      },
      {
        "label": "Audio & voice",
        "id": "mobile-nav-tab-1-variant-4",
        "href": "/api/docs/guides/audio",
        "isHome": "false"
      },
      {
        "label": "Production",
        "id": "mobile-nav-tab-1-variant-5",
        "href": "/api/docs/guides/production-best-practices",
        "isHome": "false"
      },
      {
        "label": "API reference",
        "id": "mobile-nav-tab-1-variant-6",
        "href": "/api/reference/overview",
        "isHome": "false"
      }
    ],
    "variants": [
      {
        "id": "mobile-nav-tab-1-variant-0",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/api/docs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Get started",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Quickstart",
            "href": "https://developers.openai.com/api/docs/quickstart",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Using GPT-6",
            "href": "https://developers.openai.com/api/docs/guides/latest-model",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Key concepts",
            "href": "https://developers.openai.com/api/docs/concepts",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Core concepts",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Responses API",
            "href": "https://developers.openai.com/api/docs/guides/migrate-to-responses",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Conversation state",
            "href": "https://developers.openai.com/api/docs/guides/conversation-state",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Background mode",
            "href": "https://developers.openai.com/api/docs/guides/background",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Streaming",
            "href": "https://developers.openai.com/api/docs/guides/streaming-responses",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "WebSocket mode",
            "href": "https://developers.openai.com/api/docs/guides/websocket-mode",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Mid-turn steering",
            "href": "https://developers.openai.com/api/docs/guides/steering",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Multi-agent",
            "href": "https://developers.openai.com/api/docs/guides/responses-multi-agent",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Webhooks",
            "href": "https://developers.openai.com/api/docs/guides/webhooks",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "File inputs",
            "href": "https://developers.openai.com/api/docs/guides/file-inputs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Compaction",
            "href": "https://developers.openai.com/api/docs/guides/compaction",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Counting tokens",
            "href": "https://developers.openai.com/api/docs/guides/token-counting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "SDKs and CLI",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "OpenAI SDK",
            "href": "https://developers.openai.com/api/docs/libraries",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "OpenAI CLI",
            "href": "https://developers.openai.com/api/docs/libraries/openai-cli",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Resources",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Changelog",
            "href": "https://developers.openai.com/api/docs/changelog",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Deprecations",
            "href": "https://developers.openai.com/api/docs/deprecations",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Supported countries",
            "href": "https://developers.openai.com/api/docs/supported-countries",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "OpenAI Crawlers",
            "href": "https://developers.openai.com/api/docs/bots",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Terms and policies",
            "href": "https://openai.com/policies",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "heading",
            "text": "Legacy APIs",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/agent-builder",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Migration guide",
            "href": "https://developers.openai.com/api/docs/guides/agent-builder/migrate-from-agent-builder",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Node reference",
            "href": "https://developers.openai.com/api/docs/guides/node-reference",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Safety in building agents",
            "href": "https://developers.openai.com/api/docs/guides/agent-builder-safety",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Getting started",
            "href": "https://developers.openai.com/api/docs/guides/evaluation-getting-started",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Working with evals",
            "href": "https://developers.openai.com/api/docs/guides/evals",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompt optimizer",
            "href": "https://developers.openai.com/api/docs/guides/prompt-optimizer",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "External models",
            "href": "https://developers.openai.com/api/docs/guides/external-models",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Best practices",
            "href": "https://developers.openai.com/api/docs/guides/evaluation-best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Graders",
            "href": "https://developers.openai.com/api/docs/guides/graders",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Optimization cycle",
            "href": "https://developers.openai.com/api/docs/guides/model-optimization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Supervised fine-tuning",
            "href": "https://developers.openai.com/api/docs/guides/supervised-fine-tuning",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Vision fine-tuning",
            "href": "https://developers.openai.com/api/docs/guides/vision-fine-tuning",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Direct preference optimization",
            "href": "https://developers.openai.com/api/docs/guides/direct-preference-optimization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Reinforcement fine-tuning",
            "href": "https://developers.openai.com/api/docs/guides/reinforcement-fine-tuning",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "RFT use cases",
            "href": "https://developers.openai.com/api/docs/guides/rft-use-cases",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Best practices",
            "href": "https://developers.openai.com/api/docs/guides/fine-tuning-best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Migration guide",
            "href": "https://developers.openai.com/api/docs/assistants/migration",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-1",
        "entries": [
          {
            "kind": "link",
            "text": "Model catalog",
            "href": "https://developers.openai.com/api/docs/models",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Choose a model",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Pricing",
            "href": "https://developers.openai.com/api/docs/pricing",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Model selection",
            "href": "https://developers.openai.com/api/docs/guides/model-selection",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Text and code",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Text generation",
            "href": "https://developers.openai.com/api/docs/guides/text",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Code generation",
            "href": "https://developers.openai.com/api/docs/guides/code-generation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Structured output",
            "href": "https://developers.openai.com/api/docs/guides/structured-outputs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Prompting",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/prompting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompt engineering",
            "href": "https://developers.openai.com/api/docs/guides/prompt-engineering",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Citation formatting",
            "href": "https://developers.openai.com/api/docs/guides/citation-formatting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Migration guide",
            "href": "https://developers.openai.com/api/docs/guides/prompting/migrate-from-prompt-object",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompt generation",
            "href": "https://developers.openai.com/api/docs/guides/prompt-generation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Frontend prompting",
            "href": "https://developers.openai.com/api/docs/guides/frontend-prompt",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Reasoning",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Reasoning models",
            "href": "https://developers.openai.com/api/docs/guides/reasoning",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Reasoning best practices",
            "href": "https://developers.openai.com/api/docs/guides/reasoning-best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Images",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Images and vision",
            "href": "https://developers.openai.com/api/docs/guides/images-vision",
            "active": false,
            "classes": "flex-1 "
          },
          {
            "kind": "link",
            "text": "Image input cost calculator",
            "href": "https://developers.openai.com/api/docs/guides/image-cost-calculator",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Image generation",
            "href": "https://developers.openai.com/api/docs/guides/image-generation",
            "active": false,
            "classes": "flex-1 "
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/image-generation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Image prompting",
            "href": "https://developers.openai.com/api/docs/guides/image-prompting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Realtime and audio",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Audio and speech",
            "href": "https://developers.openai.com/api/docs/guides/audio",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Getting started",
            "href": "https://developers.openai.com/api/docs/guides/realtime",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Voice agents",
            "href": "https://developers.openai.com/api/docs/guides/voice-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Specialized models",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Deep research",
            "href": "https://developers.openai.com/api/docs/guides/deep-research",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Embeddings",
            "href": "https://developers.openai.com/api/docs/guides/embeddings",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Moderation",
            "href": "https://developers.openai.com/api/docs/guides/moderation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-2",
        "entries": [
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Agents API",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/overview",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Quickstart",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/quickstart",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Architecture",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/architecture",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Configuring Agents",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/configuration",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Run and continue sessions",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/sessions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Events and items",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/sessions/events",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Manage sessions",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/sessions/manage",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Webhooks",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/sessions/webhooks",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "OpenAI-hosted sandboxes",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/environments/openai-hosted",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Self-hosted sandboxes",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/environments/self-hosted",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Sandbox lifecycle",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/environments/lifecycle",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Sandbox security",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/environments/security",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Files and artifacts",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/environments/files",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Web search",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/web-search",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Computer use",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/computer-use",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Functions",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/functions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "MCP connections",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/mcp",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Plugins",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/plugins",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Vaults",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tools/vaults",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Multi-agent",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/multi-agent",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Observability and usage",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/observability",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Tracing",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/tracing",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Errors and recovery",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/errors",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "API reference",
            "href": "https://developers.openai.com/api/reference/resources/beta/subresources/agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Bedrock Managed Agents",
            "href": "https://developers.openai.com/api/docs/guides/agents-api/bedrock-managed-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Agents SDK",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/agents/sdk",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Quickstart",
            "href": "https://developers.openai.com/api/docs/guides/agents/quickstart",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Agent definitions",
            "href": "https://developers.openai.com/api/docs/guides/agents/define-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Models and providers",
            "href": "https://developers.openai.com/api/docs/guides/agents/models",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Running agents",
            "href": "https://developers.openai.com/api/docs/guides/agents/running-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Sandbox agents",
            "href": "https://developers.openai.com/api/docs/guides/agents/sandboxes",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Orchestration",
            "href": "https://developers.openai.com/api/docs/guides/agents/orchestration",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Guardrails",
            "href": "https://developers.openai.com/api/docs/guides/agents/guardrails-approvals",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Results and state",
            "href": "https://developers.openai.com/api/docs/guides/agents/results",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Integrations and observability",
            "href": "https://developers.openai.com/api/docs/guides/agents/integrations-observability",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Evaluate agent workflows",
            "href": "https://developers.openai.com/api/docs/guides/agent-evals",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "ChatKit",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/chatkit",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Customize",
            "href": "https://developers.openai.com/api/docs/guides/chatkit-themes",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Widgets",
            "href": "https://developers.openai.com/api/docs/guides/chatkit-widgets",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Actions",
            "href": "https://developers.openai.com/api/docs/guides/chatkit-actions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Advanced integrations",
            "href": "https://developers.openai.com/api/docs/guides/custom-chatkit",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-3",
        "entries": [
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/tools",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Function calling",
            "href": "https://developers.openai.com/api/docs/guides/function-calling",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Search and retrieval",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Web search",
            "href": "https://developers.openai.com/api/docs/guides/tools-web-search",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "File search",
            "href": "https://developers.openai.com/api/docs/guides/tools-file-search",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Retrieval",
            "href": "https://developers.openai.com/api/docs/guides/retrieval",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Connect tools and data",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "MCP servers",
            "href": "https://developers.openai.com/api/docs/guides/tools-connectors-mcp",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Secure MCP Tunnel",
            "href": "https://developers.openai.com/api/docs/guides/secure-mcp-tunnels",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Build tool workflows",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Skills",
            "href": "https://developers.openai.com/api/docs/guides/tools-skills",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Tool search",
            "href": "https://developers.openai.com/api/docs/guides/tools-tool-search",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Programmatic tool calling",
            "href": "https://developers.openai.com/api/docs/guides/tools-programmatic-tool-calling",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Async tool calling",
            "href": "https://developers.openai.com/api/docs/guides/async-tool-calling",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Computer and code",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Shell",
            "href": "https://developers.openai.com/api/docs/guides/tools-shell",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Computer use",
            "href": "https://developers.openai.com/api/docs/guides/tools-computer-use",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Apply Patch",
            "href": "https://developers.openai.com/api/docs/guides/tools-apply-patch",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Local shell",
            "href": "https://developers.openai.com/api/docs/guides/tools-local-shell",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Code interpreter",
            "href": "https://developers.openai.com/api/docs/guides/tools-code-interpreter",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Media",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Image generation",
            "href": "https://developers.openai.com/api/docs/guides/tools-image-generation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-4",
        "entries": [
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/audio",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "GPT-Live",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Getting started",
            "href": "https://developers.openai.com/api/docs/guides/live",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompting",
            "href": "https://developers.openai.com/api/docs/guides/live-prompting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Managing sessions",
            "href": "https://developers.openai.com/api/docs/guides/live-conversations",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Delegation and tools",
            "href": "https://developers.openai.com/api/docs/guides/live-delegation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Migrate to GPT-Live",
            "href": "https://developers.openai.com/api/docs/guides/live-migration",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Partner integrations",
            "href": "https://developers.openai.com/api/docs/guides/live-partner-integrations",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Realtime API",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Getting started",
            "href": "https://developers.openai.com/api/docs/guides/realtime",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompting",
            "href": "https://developers.openai.com/api/docs/guides/voice-prompting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Managing conversations",
            "href": "https://developers.openai.com/api/docs/guides/realtime-conversations",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Voice activity detection",
            "href": "https://developers.openai.com/api/docs/guides/realtime-vad",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Tools and MCP",
            "href": "https://developers.openai.com/api/docs/guides/realtime-mcp",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Build with voice",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Voice agents",
            "href": "https://developers.openai.com/api/docs/guides/voice-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Custom voices",
            "href": "https://developers.openai.com/api/docs/guides/custom-voices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Cost optimization",
            "href": "https://developers.openai.com/api/docs/guides/voice-latency-cost",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Connections",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "WebRTC",
            "href": "https://developers.openai.com/api/docs/guides/voice-webrtc",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "WebRTC with WARP",
            "href": "https://developers.openai.com/api/docs/guides/realtime-webrtc-warp",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "WebSockets",
            "href": "https://developers.openai.com/api/docs/guides/voice-websockets",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Telephony and SIP",
            "href": "https://developers.openai.com/api/docs/guides/voice-sip",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Server-side controls",
            "href": "https://developers.openai.com/api/docs/guides/voice-server-controls",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Audio processing",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "File transcription",
            "href": "https://developers.openai.com/api/docs/guides/speech-to-text",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Live transcription",
            "href": "https://developers.openai.com/api/docs/guides/realtime-transcription",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Live translation",
            "href": "https://developers.openai.com/api/docs/guides/realtime-translation",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Text to speech",
            "href": "https://developers.openai.com/api/docs/guides/text-to-speech",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Audio in Chat Completions",
            "href": "https://developers.openai.com/api/docs/guides/audio-chat-completions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-5",
        "entries": [
          {
            "kind": "heading",
            "text": "Go live",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Production best practices",
            "href": "https://developers.openai.com/api/docs/guides/production-best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Deployment checklist",
            "href": "https://developers.openai.com/api/docs/guides/deployment-checklist",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Performance and quality",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Fast mode",
            "href": "https://developers.openai.com/api/docs/guides/fast-mode",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Ultrafast mode",
            "href": "https://developers.openai.com/api/docs/guides/ultrafast-mode",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Latency optimization",
            "href": "https://developers.openai.com/api/docs/guides/latency-optimization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Predicted Outputs",
            "href": "https://developers.openai.com/api/docs/guides/predicted-outputs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Accuracy optimization",
            "href": "https://developers.openai.com/api/docs/guides/optimizing-llm-accuracy",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Cost and throughput",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Cost optimization",
            "href": "https://developers.openai.com/api/docs/guides/cost-optimization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Prompt caching",
            "href": "https://developers.openai.com/api/docs/guides/prompt-caching",
            "active": false,
            "classes": "flex-1 "
          },
          {
            "kind": "link",
            "text": "Prompt cache diagnostics",
            "href": "https://developers.openai.com/api/docs/guides/prompt-caching/diagnostics",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Batch",
            "href": "https://developers.openai.com/api/docs/guides/batch",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Flex processing",
            "href": "https://developers.openai.com/api/docs/guides/flex-processing",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Safety and governance",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Safety best practices",
            "href": "https://developers.openai.com/api/docs/guides/safety-best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Red teaming",
            "href": "https://developers.openai.com/api/docs/guides/red-teaming",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Daybreak",
            "href": "https://developers.openai.com/api/docs/guides/daybreak",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Safety classifiers",
            "href": "https://developers.openai.com/api/docs/guides/safety-checks",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Cybersecurity checks",
            "href": "https://developers.openai.com/api/docs/guides/safety-checks/cybersecurity",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Misalignment monitoring",
            "href": "https://developers.openai.com/api/docs/guides/safety-checks/misalignment-monitoring",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Enforcement notifications",
            "href": "https://developers.openai.com/api/docs/guides/safety-enforcement",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Under-18 guidance",
            "href": "https://developers.openai.com/api/docs/guides/safety-checks/under-18-api-guidance",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "CSAM guidance",
            "href": "https://developers.openai.com/api/docs/guides/csam-guidance",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Content provenance",
            "href": "https://developers.openai.com/api/docs/guides/content-provenance",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Your data",
            "href": "https://developers.openai.com/api/docs/guides/your-data",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Private Safety Processing",
            "href": "https://developers.openai.com/api/docs/guides/private-safety-processing",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Permissions",
            "href": "https://developers.openai.com/api/docs/guides/rbac",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Infrastructure and access",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Terraform provider",
            "href": "https://developers.openai.com/api/docs/guides/terraform",
            "active": false,
            "classes": "flex-1 "
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/api/docs/guides/terraform",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Projects and access",
            "href": "https://developers.openai.com/api/docs/guides/terraform/projects-and-access",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Service accounts",
            "href": "https://developers.openai.com/api/docs/guides/terraform/service-accounts",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Rate limits and spend",
            "href": "https://developers.openai.com/api/docs/guides/terraform/rate-limits-and-spend",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Model, tool, and data controls",
            "href": "https://developers.openai.com/api/docs/guides/terraform/project-controls",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Import and reconciliation",
            "href": "https://developers.openai.com/api/docs/guides/terraform/import-and-reconcile",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Private Link",
            "href": "https://developers.openai.com/api/docs/guides/private-link",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "IP allowlist",
            "href": "https://developers.openai.com/api/docs/guides/ip-allowlist",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Organization blocking",
            "href": "https://developers.openai.com/api/docs/guides/organization-blocking",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Mutual TLS",
            "href": "https://developers.openai.com/api/docs/guides/mutual-tls",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Workload identity federation",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation",
            "active": false,
            "classes": "flex-1 "
          },
          {
            "kind": "link",
            "text": "Federation rules",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/federation-rules",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "X.509 certificates",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/x509",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Kubernetes",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/kubernetes",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "AWS",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/aws",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Microsoft Azure",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/microsoft-azure",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Google Cloud",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/google-cloud",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Oracle Cloud Infrastructure",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/oracle-cloud",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "GitHub Actions",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/github-actions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "SPIFFE",
            "href": "https://developers.openai.com/api/docs/guides/workload-identity-federation/spiffe",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "IP egress ranges",
            "href": "https://developers.openai.com/api/docs/guides/ip-addresses",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Amazon Bedrock",
            "href": "https://developers.openai.com/api/docs/guides/amazon-bedrock",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Operations",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Rate limits",
            "href": "https://developers.openai.com/api/docs/guides/rate-limits",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Spend limits",
            "href": "https://developers.openai.com/api/docs/guides/spend-limits",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Admin APIs",
            "href": "https://developers.openai.com/api/docs/guides/admin-apis",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Error codes",
            "href": "https://developers.openai.com/api/docs/guides/error-codes",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-1-variant-6",
        "entries": []
      }
    ],
    "label": "API"
  },
  {
    "id": "mobile-nav-tab-2",
    "defaultVariant": "mobile-nav-tab-2-variant-0",
    "options": [
      {
        "label": "Sign in with ChatGPT",
        "id": "mobile-nav-tab-2-variant-1",
        "href": "/siwc",
        "isHome": "false"
      },
      {
        "label": "Plugins",
        "id": "mobile-nav-tab-2-variant-2",
        "href": "/plugins",
        "isHome": "false"
      },
      {
        "label": "Workspace Agents",
        "id": "mobile-nav-tab-2-variant-3",
        "href": "/workspace-agents",
        "isHome": "false"
      },
      {
        "label": "Commerce",
        "id": "mobile-nav-tab-2-variant-4",
        "href": "/commerce",
        "isHome": "false"
      },
      {
        "label": "Ads",
        "id": "mobile-nav-tab-2-variant-5",
        "href": "/ads",
        "isHome": "false"
      }
    ],
    "variants": [
      {
        "id": "mobile-nav-tab-2-variant-0",
        "entries": []
      },
      {
        "id": "mobile-nav-tab-2-variant-1",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/siwc",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Quickstart",
            "href": "https://developers.openai.com/siwc/quickstart",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Request a client ID",
            "href": "https://developers.openai.com/siwc/request-client-id",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Identity",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "On your website",
            "href": "https://developers.openai.com/siwc/website",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "In your ChatGPT plugin",
            "href": "https://developers.openai.com/siwc/chatgpt-plugin",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "ChatGPT plan usage",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "UI/UX guidelines",
            "href": "https://developers.openai.com/siwc/ui-ux-guidelines",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Registration and sign-in",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/sign-in",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Accounts and sessions",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Models and inference",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex app-server",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Self-hosted VMs",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/self-hosted-vms",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Token reference",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/token-reference",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Errors and recovery",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Preview limitations",
            "href": "https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-2-variant-2",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/plugins",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Quickstart",
            "href": "https://developers.openai.com/plugins/quickstart",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Core concepts",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Plugin architecture",
            "href": "https://developers.openai.com/plugins/concepts/plugins",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Skills",
            "href": "https://developers.openai.com/plugins/concepts/skills",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "MCP server",
            "href": "https://developers.openai.com/plugins/concepts/mcp-server",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Plan",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Brainstorm use cases",
            "href": "https://developers.openai.com/plugins/plan/use-case",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Define tools",
            "href": "https://developers.openai.com/plugins/plan/tools",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Build",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Build an MCP server",
            "href": "https://developers.openai.com/plugins/build/mcp-server",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Add UI to your MCP server (optional)",
            "href": "https://developers.openai.com/plugins/build/chatgpt-ui",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Add events to your MCP server (optional)",
            "href": "https://developers.openai.com/plugins/build/mcp-events",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Extensions",
            "href": "https://developers.openai.com/plugins/build/extensions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Authenticate users",
            "href": "https://developers.openai.com/plugins/build/auth",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Build skills",
            "href": "https://developers.openai.com/plugins/build/skills",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Package your plugin",
            "href": "https://developers.openai.com/plugins/build/plugins",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Examples",
            "href": "https://developers.openai.com/plugins/build/examples",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Test and publish",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Connect and test your plugin",
            "href": "https://developers.openai.com/plugins/deploy/connect-chatgpt",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Submit and publish",
            "href": "https://developers.openai.com/plugins/deploy/submission",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Submission error reference",
            "href": "https://developers.openai.com/plugins/deploy/submission-errors",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Conversion specs",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Restaurant reservation spec",
            "href": "https://developers.openai.com/plugins/guides/restaurant-reservation-conversion-spec",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Get Quote spec",
            "href": "https://developers.openai.com/plugins/guides/local-services-request-quote-conversion-spec",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Product checkout spec",
            "href": "https://developers.openai.com/plugins/guides/product-checkout-conversion-spec",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Guides",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "UI guidelines",
            "href": "https://developers.openai.com/plugins/concepts/ui-guidelines",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Optimize Metadata",
            "href": "https://developers.openai.com/plugins/guides/optimize-metadata",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Submit a Claude Code plugin",
            "href": "https://developers.openai.com/plugins/guides/submit-claude-plugin",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Security & Privacy",
            "href": "https://developers.openai.com/plugins/guides/security-privacy",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Troubleshooting",
            "href": "https://developers.openai.com/plugins/deploy/troubleshooting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Resources",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Changelog",
            "href": "https://developers.openai.com/plugins/changelog",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Plugin guidelines",
            "href": "https://developers.openai.com/plugins/plugin-guidelines",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "MCP server review requirements",
            "href": "https://developers.openai.com/plugins/deploy/app-review",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Plugin UI reference",
            "href": "https://developers.openai.com/plugins/reference",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Checkout API reference",
            "href": "https://developers.openai.com/plugins/build/monetization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-2-variant-3",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/workspace-agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Get started",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Trigger workspace agent runs",
            "href": "https://developers.openai.com/workspace-agents/trigger-runs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Authenticate with Workspace Agent access tokens",
            "href": "https://developers.openai.com/workspace-agents/authentication",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-2-variant-4",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/commerce",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Guides",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Get started",
            "href": "https://developers.openai.com/commerce/guides/get-started",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Best practices",
            "href": "https://developers.openai.com/commerce/guides/best-practices",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "File Upload",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/commerce/specs/file-upload/overview",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Products",
            "href": "https://developers.openai.com/commerce/specs/file-upload/products",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "API",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/commerce/specs/api/overview",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Feeds",
            "href": "https://developers.openai.com/commerce/specs/api/feeds",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Products",
            "href": "https://developers.openai.com/commerce/specs/api/products",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Promotions",
            "href": "https://developers.openai.com/commerce/specs/api/promotions",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-2-variant-5",
        "entries": [
          {
            "kind": "link",
            "text": "Ads Overview",
            "href": "https://developers.openai.com/ads",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Measurement",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Measurement Pixel",
            "href": "https://developers.openai.com/ads/measurement-pixel",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Multiple Pixels (Advanced)",
            "href": "https://developers.openai.com/ads/multiple-pixels",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Image Tag",
            "href": "https://developers.openai.com/ads/image-tag",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Conversions API",
            "href": "https://developers.openai.com/ads/conversions-api",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Supported Events",
            "href": "https://developers.openai.com/ads/supported-events",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Advertiser API",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Overview",
            "href": "https://developers.openai.com/ads/api-overview",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "API Partner Setup",
            "href": "https://developers.openai.com/ads/api-partner-setup",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Campaign Management",
            "href": "https://developers.openai.com/ads/campaign-management",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Bidding & Budgets",
            "href": "https://developers.openai.com/ads/bidding-and-budgets",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Targeting",
            "href": "https://developers.openai.com/ads/campaign-targeting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Product Feeds",
            "href": "https://developers.openai.com/ads/product-feeds",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Hotel Feeds (limited beta)",
            "href": "https://developers.openai.com/ads/hotel-feeds",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Conversion Tracking",
            "href": "https://developers.openai.com/ads/conversion-tracking",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Reporting",
            "href": "https://developers.openai.com/ads/reporting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Troubleshooting",
            "href": "https://developers.openai.com/ads/troubleshooting",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Account Management",
            "href": "https://developers.openai.com/ads/account-management",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "API Reference",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Authentication",
            "href": "https://developers.openai.com/ads/api-reference/authentication",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Ad Account",
            "href": "https://developers.openai.com/ads/api-reference/ad-account",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Audit Logs",
            "href": "https://developers.openai.com/ads/api-reference/audit-logs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Campaigns",
            "href": "https://developers.openai.com/ads/api-reference/campaigns",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Ad Groups",
            "href": "https://developers.openai.com/ads/api-reference/ad-groups",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Ads",
            "href": "https://developers.openai.com/ads/api-reference/ads",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Insights",
            "href": "https://developers.openai.com/ads/api-reference/insights",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Files",
            "href": "https://developers.openai.com/ads/api-reference/files",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Conversion Setup",
            "href": "https://developers.openai.com/ads/api-reference/conversion-setup",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-2-variant-6",
        "entries": []
      },
      {
        "id": "mobile-nav-tab-2-variant-7",
        "entries": []
      }
    ],
    "label": "ChatGPT"
  },
  {
    "id": "mobile-nav-tab-7",
    "defaultVariant": "mobile-nav-tab-7-variant-2",
    "options": [
      {
        "label": "Blog",
        "id": "mobile-nav-tab-7-variant-2",
        "href": "/blog",
        "isHome": "false"
      },
      {
        "label": "Cookbook",
        "id": "mobile-nav-tab-7-variant-3",
        "href": "/cookbook",
        "isHome": "false"
      },
      {
        "label": "Learn",
        "id": "mobile-nav-tab-7-variant-4",
        "href": "/learn",
        "isHome": "false"
      },
      {
        "label": "Community",
        "id": "mobile-nav-tab-7-variant-5",
        "href": "/community",
        "isHome": "false"
      }
    ],
    "variants": [
      {
        "id": "mobile-nav-tab-7-variant-0",
        "entries": []
      },
      {
        "id": "mobile-nav-tab-7-variant-1",
        "entries": []
      },
      {
        "id": "mobile-nav-tab-7-variant-2",
        "entries": [
          {
            "kind": "link",
            "text": "All posts",
            "href": "/",
            "active": true,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block text-default bg-primary-ghost-active "
          },
          {
            "kind": "heading",
            "text": "Recent",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Bringing my LED display to life with GPT-Live-1 and Codex",
            "href": "https://developers.openai.com/blog/bringing-my-led-display-to-life",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Rethinking skills and prompts for GPT-6 Astra",
            "href": "https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Architectural visualization with Astra",
            "href": "https://developers.openai.com/blog/architectural-visualization-with-astra",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Building games with Astra",
            "href": "https://developers.openai.com/blog/how-to-build-games-with-astra",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Meet Rosalind Workbench: Empowering every scientist to be their own research team",
            "href": "https://developers.openai.com/blog/rosalind-workbench",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Topics",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "General",
            "href": "https://developers.openai.com/blog/topic/general",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "API",
            "href": "https://developers.openai.com/blog/topic/api",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Apps SDK",
            "href": "https://developers.openai.com/blog/topic/apps-sdk",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Audio",
            "href": "https://developers.openai.com/blog/topic/audio",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex",
            "href": "https://developers.openai.com/blog/topic/codex",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Life sciences",
            "href": "https://developers.openai.com/blog/topic/life-sciences",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-7-variant-3",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/cookbook",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Topics",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Sign-in with ChatGPT",
            "href": "https://developers.openai.com/cookbook/topic/sign-in-with-chatgpt",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Agents",
            "href": "https://developers.openai.com/cookbook/topic/agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Evals",
            "href": "https://developers.openai.com/cookbook/topic/evals",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Multimodal",
            "href": "https://developers.openai.com/cookbook/topic/multimodal",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Text",
            "href": "https://developers.openai.com/cookbook/topic/text",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Guardrails",
            "href": "https://developers.openai.com/cookbook/topic/guardrails",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Optimization",
            "href": "https://developers.openai.com/cookbook/topic/optimization",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "ChatGPT",
            "href": "https://developers.openai.com/cookbook/topic/chatgpt",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex",
            "href": "https://developers.openai.com/cookbook/topic/codex",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "gpt-oss",
            "href": "https://developers.openai.com/cookbook/topic/gpt-oss",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Contribute",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Cookbook on GitHub",
            "href": "https://github.com/openai/openai-cookbook",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          }
        ]
      },
      {
        "id": "mobile-nav-tab-7-variant-4",
        "entries": [
          {
            "kind": "link",
            "text": "Home",
            "href": "https://developers.openai.com/learn",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "OpenAI Developers plugin",
            "href": "https://developers.openai.com/learn/developers-codex-plugin",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Docs MCP",
            "href": "https://developers.openai.com/learn/docs-mcp",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Categories",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Demo apps",
            "href": "https://developers.openai.com/learn/code",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Videos",
            "href": "https://developers.openai.com/learn/videos",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Topics",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Agents",
            "href": "https://developers.openai.com/learn/agents",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Audio & Voice",
            "href": "https://developers.openai.com/learn/audio",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Computer Use",
            "href": "https://developers.openai.com/learn/cua",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex",
            "href": "https://developers.openai.com/learn/codex",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Evals",
            "href": "https://developers.openai.com/learn/evals",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "gpt-oss",
            "href": "https://developers.openai.com/learn/gpt-oss",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Fine-tuning",
            "href": "https://developers.openai.com/learn/fine-tuning",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Image generation",
            "href": "https://developers.openai.com/learn/imagegen",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Scaling",
            "href": "https://developers.openai.com/learn/scaling",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Tools",
            "href": "https://developers.openai.com/learn/tools",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Video generation",
            "href": "https://developers.openai.com/learn/videogen",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          }
        ]
      },
      {
        "id": "mobile-nav-tab-7-variant-5",
        "entries": [
          {
            "kind": "link",
            "text": "Community",
            "href": "https://developers.openai.com/community",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "heading",
            "text": "Programs",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Codex Ambassadors",
            "href": "https://developers.openai.com/community/codex-ambassadors",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex for Students",
            "href": "https://developers.openai.com/community/students",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "Codex for Open Source",
            "href": "https://developers.openai.com/community/codex-for-oss",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover "
          },
          {
            "kind": "link",
            "text": "OpenAI for Startups",
            "href": "https://openai.com/business/why-openai/startups/",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "heading",
            "text": "Spaces",
            "href": null,
            "active": false,
            "classes": "text-xs tracking-wide text-secondary"
          },
          {
            "kind": "link",
            "text": "Events",
            "href": "https://luma.com/codex-community?utm_source=oaidevs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "link",
            "text": "Developer Forum",
            "href": "https://community.openai.com/",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "link",
            "text": "Discord",
            "href": "https://discord.com/invite/openai",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "link",
            "text": "Reddit",
            "href": "https://www.reddit.com/r/OpenAI/",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          },
          {
            "kind": "link",
            "text": "X",
            "href": "https://x.com/OpenAIDevs",
            "active": false,
            "classes": "px-3 py-1.5 rounded-lg transition-colors block hover:text-default hover:bg-primary-ghost-hover flex items-center justify-between gap-2"
          }
        ]
      }
    ],
    "label": "Resources"
  }
];
