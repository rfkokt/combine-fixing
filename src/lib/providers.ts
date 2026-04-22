export interface ProviderConfig {
  id: string;
  name: string;
  baseUrl: string;
  models: string[];
}

export const PRESET_PROVIDERS: ProviderConfig[] = [
  {
    id: 'groq',
    name: 'Groq (Fast & Free)',
    baseUrl: 'https://api.groq.com/openai/v1',
    models: [
      'llama-3.3-70b-versatile',
      'llama-3.1-8b-instant',
      'gemma2-9b-it',
      'mixtral-8x7b-32768'
    ]
  },
  {
    id: 'google',
    name: 'Google AI Studio (Free)',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
    models: [
      'gemini-2.5-flash',
      'gemini-2.0-flash',
      'gemini-2.0-flash-lite',
      'gemini-1.5-flash',
      'gemini-1.5-pro'
    ]
  },
  {
    id: 'openai',
    name: 'OpenAI',
    baseUrl: 'https://api.openai.com/v1',
    models: [
      'gpt-4o',
      'gpt-4o-mini',
      'o3-mini',
      'o1',
      'o1-mini',
      'gpt-3.5-turbo'
    ]
  },
  {
    id: 'minimax',
    name: 'MiniMax',
    baseUrl: 'https://api.minimax.io/v1/text/chatcompletion_v2',
    models: [
      'MiniMax-M2.7',
      'MiniMax-M2.7-highspeed',
      'MiniMax-M2.5',
      'MiniMax-M2.5-highspeed',
      'MiniMax-M2-her',
      'MiniMax-M2.1',
      'MiniMax-M2'
    ]
  },
  {
    id: 'zai',
    name: 'Z.ai (Zhipu AI)',
    baseUrl: 'https://api.z.ai/api/coding/paas/v4',
    models: [
      'glm-4.7',
      'glm-4.6',
      'glm-4.5-Flash',
      'glm-4.5-Air',
      'glm-4.5-X',
      'glm-5.1',
      'glm-5',
      'glm-z1-flash',
      'glm-z1-air',
      'glm-z1-airx'
    ]
  },
  {
    id: 'custom',
    name: 'Custom / Other',
    baseUrl: '',
    models: []
  }
];
