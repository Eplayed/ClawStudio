// costCalculator.ts - API cost calculation engine
// Calculates precise token costs for various LLM providers

export interface TokenUsage {
  input: number
  output: number
  image: number // screenshot tokens
}

interface ModelPricing {
  input: number   // per 1M tokens
  output: number  // per 1M tokens
  image: number   // per 1M tokens (vision/screenshot)
}

// Model pricing table (USD per 1M tokens, as of 2024)
const PRICING: Record<string, ModelPricing> = {
  // Anthropic models
  'claude-3-5-sonnet-20241022': { input: 3.0, output: 15.0, image: 3.84 },
  'claude-3-5-sonnet-latest':   { input: 3.0, output: 15.0, image: 3.84 },
  'claude-3-opus-20240229':    { input: 15.0, output: 75.0, image: 3.84 },
  'claude-3-haiku-20240307':   { input: 0.25, output: 1.25, image: 0.25 },
  'claude-3-sonnet-20240229':  { input: 3.0, output: 15.0, image: 3.84 },
  // OpenAI models
  'gpt-4o':          { input: 2.5, output: 10.0, image: 3.75 },
  'gpt-4o-mini':    { input: 0.15, output: 0.60, image: 0.60 },
  'gpt-4-turbo':    { input: 10.0, output: 30.0, image: 10.0 },
  'gpt-4':          { input: 30.0, output: 60.0, image: 30.0 },
  // DeepSeek
  'deepseek-chat':  { input: 0.27, output: 1.10, image: 0.27 },
  // Default fallback
  'default':        { input: 3.0, output: 15.0, image: 3.0 },
}

/**
 * Calculate the cost in USD for a given model and token usage.
 * Returns a number precise to 6 decimal places.
 */
export function calculateCost(model: string, tokens: TokenUsage): number {
  const pricing = PRICING[model] || PRICING['default']
  const inputCost = (tokens.input / 1_000_000) * pricing.input
  const outputCost = (tokens.output / 1_000_000) * pricing.output
  const imageCost = (tokens.image / 1_000_000) * pricing.image
  return Math.round((inputCost + outputCost + imageCost) * 1_000_000) / 1_000_000
}

/**
 * Calculate screenshot tokens based on image dimensions.
 * Uses Claude's image token formula: ceil(width/10) * ceil(height/10) * 85/12 + 85
 */
export function estimateScreenshotTokens(width: number, height: number): number {
  // Claude's formula for image token count
  const tilesX = Math.ceil(width / 512)
  const tilesY = Math.ceil(height / 512)
  const baseTokens = 85
  const tileTokens = tilesX * tilesY * 170
  return baseTokens + tileTokens
}

/**
 * Estimate cost of a single screenshot at standard resolution
 */
export function estimateScreenshotCost(model: string): number {
  const tokens = estimateScreenshotTokens(1280, 800)
  const pricing = PRICING[model] || PRICING['default']
  return Math.round(((tokens / 1_000_000) * pricing.image) * 1_000_000) / 1_000_000
}

/**
 * Format cost for display (e.g., $0.003456)
 */
export function formatCost(cost: number): string {
  if (cost < 0.0001) return '$' + cost.toExponential(2)
  if (cost < 0.01) return '$' + cost.toFixed(4)
  return '$' + cost.toFixed(2)
}

/**
 * Budget status: safe / warning / danger / exceeded
 */
export type BudgetStatus = 'safe' | 'warning' | 'danger' | 'exceeded'

export function getBudgetStatus(current: number, limit: number): BudgetStatus {
  if (limit <= 0) return 'safe'
  const ratio = current / limit
  if (ratio >= 1.0) return 'exceeded'
  if (ratio >= 0.8) return 'danger'
  if (ratio >= 0.5) return 'warning'
  return 'safe'
}

export function getBudgetColor(status: BudgetStatus): string {
  switch (status) {
    case 'safe':     return 'var(--green)'
    case 'warning': return 'var(--amber)'
    case 'danger':  return 'var(--red)'
    case 'exceeded':return 'var(--red)'
  }
}

export function getBudgetGradient(status: BudgetStatus): string {
  switch (status) {
    case 'safe':     return 'url(#fuelGradGreen)'
    case 'warning': return 'url(#fuelGradAmber)'
    case 'danger':  return 'url(#fuelGradRed)'
    case 'exceeded':return 'url(#fuelGradRed)'
  }
}
