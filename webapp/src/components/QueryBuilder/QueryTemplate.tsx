// Query template parameter types
export interface QueryParameter {
  name: string;
  label: string;
  type: 'text' | 'number' | 'select' | 'multiselect' | 'entity' | 'date-range';
  required: boolean;
  placeholder?: string;
  options?: string[];
  defaultValue?: unknown;
  validation?: {
    min?: number;
    max?: number;
    pattern?: string;
    message?: string;
  };
}

export interface QueryTemplate {
  id: string;
  name: string;
  description: string;
  category: 'analysis' | 'search' | 'comparison' | 'trend' | 'entity' | 'relation' | 'custom';
  template: string;
  parameters: QueryParameter[];
  examples: string[];
  tags: string[];
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  estimatedTime?: string;
}

// Predefined query templates
export const DEFAULT_QUERY_TEMPLATES: QueryTemplate[] = [
  {
    id: 'entity-analysis',
    name: 'Entity Analysis',
    description: 'Analyze mentions and sentiment around specific entities',
    category: 'entity',
    template: 'Analyze the entity "{entity}" mentioned in Hacker News discussions. Focus on {analysisType} and provide insights about {aspects}. Time range: {timeRange}.',
    parameters: [
      {
        name: 'entity',
        label: 'Entity Name',
        type: 'entity',
        required: true,
        placeholder: 'e.g., OpenAI, JavaScript, etc.',
      },
      {
        name: 'analysisType',
        label: 'Analysis Type',
        type: 'select',
        required: true,
        options: ['sentiment', 'frequency', 'context', 'relationships', 'trends'],
        defaultValue: 'sentiment',
      },
      {
        name: 'aspects',
        label: 'Focus Aspects',
        type: 'multiselect',
        required: false,
        options: ['technical discussion', 'business impact', 'user opinions', 'market position', 'innovations'],
        defaultValue: ['technical discussion', 'user opinions'],
      },
      {
        name: 'timeRange',
        label: 'Time Range',
        type: 'select',
        required: true,
        options: ['last week', 'last month', 'last 3 months', 'last year', 'all time'],
        defaultValue: 'last month',
      },
    ],
    examples: [
      'Analyze the entity "OpenAI" mentioned in Hacker News discussions. Focus on sentiment and provide insights about technical discussion and user opinions. Time range: last month.',
      'Analyze the entity "React" mentioned in Hacker News discussions. Focus on trends and provide insights about technical discussion and market position. Time range: last 3 months.',
    ],
    tags: ['entity', 'analysis', 'sentiment'],
    difficulty: 'beginner',
    estimatedTime: '2-3 minutes',
  },
  {
    id: 'trend-comparison',
    name: 'Trend Comparison',
    description: 'Compare trends between multiple technologies or entities',
    category: 'comparison',
    template: 'Compare the discussion trends between {entities} on Hacker News over {timeRange}. Analyze {metrics} and identify key differences in {comparisonAspects}.',
    parameters: [
      {
        name: 'entities',
        label: 'Entities to Compare',
        type: 'text',
        required: true,
        placeholder: 'e.g., React, Vue, Angular (comma-separated)',
        validation: {
          pattern: '^[^,]+(,[^,]+)+$',
          message: 'Please enter at least 2 entities separated by commas',
        },
      },
      {
        name: 'timeRange',
        label: 'Time Range',
        type: 'select',
        required: true,
        options: ['last month', 'last 3 months', 'last 6 months', 'last year'],
        defaultValue: 'last 3 months',
      },
      {
        name: 'metrics',
        label: 'Comparison Metrics',
        type: 'multiselect',
        required: true,
        options: ['mention frequency', 'sentiment scores', 'user engagement', 'discussion depth', 'adoption indicators'],
        defaultValue: ['mention frequency', 'sentiment scores'],
      },
      {
        name: 'comparisonAspects',
        label: 'Comparison Aspects',
        type: 'multiselect',
        required: false,
        options: ['performance', 'ease of use', 'community support', 'documentation', 'ecosystem', 'learning curve'],
        defaultValue: ['performance', 'ease of use'],
      },
    ],
    examples: [
      'Compare the discussion trends between React, Vue, Angular on Hacker News over last 3 months. Analyze mention frequency and sentiment scores and identify key differences in performance and ease of use.',
    ],
    tags: ['comparison', 'trends', 'technology'],
    difficulty: 'intermediate',
    estimatedTime: '3-5 minutes',
  },
  {
    id: 'relationship-exploration',
    name: 'Relationship Exploration',
    description: 'Explore relationships between entities and their connections',
    category: 'relation',
    template: 'Explore the relationships between {primaryEntity} and other entities in Hacker News discussions. Focus on {relationshipTypes} and analyze {networkAspects} within {scope}.',
    parameters: [
      {
        name: 'primaryEntity',
        label: 'Primary Entity',
        type: 'entity',
        required: true,
        placeholder: 'e.g., Microsoft, Python, etc.',
      },
      {
        name: 'relationshipTypes',
        label: 'Relationship Types',
        type: 'multiselect',
        required: true,
        options: ['competitive', 'collaborative', 'dependent', 'alternative', 'ecosystem', 'ownership'],
        defaultValue: ['competitive', 'collaborative'],
      },
      {
        name: 'networkAspects',
        label: 'Network Analysis',
        type: 'multiselect',
        required: false,
        options: ['connection strength', 'influence patterns', 'discussion clusters', 'co-occurrence frequency'],
        defaultValue: ['connection strength', 'co-occurrence frequency'],
      },
      {
        name: 'scope',
        label: 'Analysis Scope',
        type: 'select',
        required: true,
        options: ['direct connections only', 'one-degree separation', 'full network'],
        defaultValue: 'one-degree separation',
      },
    ],
    examples: [
      'Explore the relationships between Microsoft and other entities in Hacker News discussions. Focus on competitive and collaborative and analyze connection strength and co-occurrence frequency within one-degree separation.',
    ],
    tags: ['relationships', 'network', 'entities'],
    difficulty: 'advanced',
    estimatedTime: '5-7 minutes',
  },
  {
    id: 'topic-deep-dive',
    name: 'Topic Deep Dive',
    description: 'Deep analysis of specific topics or discussions',
    category: 'analysis',
    template: 'Perform a deep dive analysis on {topic} discussions in Hacker News. Examine {analysisDepth} and highlight {insights}. Filter by {filters}.',
    parameters: [
      {
        name: 'topic',
        label: 'Topic/Theme',
        type: 'text',
        required: true,
        placeholder: 'e.g., machine learning, startup funding, privacy',
      },
      {
        name: 'analysisDepth',
        label: 'Analysis Depth',
        type: 'multiselect',
        required: true,
        options: ['key themes', 'expert opinions', 'controversial points', 'emerging trends', 'community consensus'],
        defaultValue: ['key themes', 'expert opinions'],
      },
      {
        name: 'insights',
        label: 'Insight Focus',
        type: 'multiselect',
        required: false,
        options: ['practical applications', 'future predictions', 'current challenges', 'success stories', 'best practices'],
        defaultValue: ['practical applications', 'current challenges'],
      },
      {
        name: 'filters',
        label: 'Content Filters',
        type: 'multiselect',
        required: false,
        options: ['high-quality posts only', 'expert contributors', 'recent discussions', 'popular posts', 'controversial posts'],
        defaultValue: ['high-quality posts only'],
      },
    ],
    examples: [
      'Perform a deep dive analysis on machine learning discussions in Hacker News. Examine key themes and expert opinions and highlight practical applications and current challenges. Filter by high-quality posts only.',
    ],
    tags: ['analysis', 'deep-dive', 'topics'],
    difficulty: 'intermediate',
    estimatedTime: '4-6 minutes',
  },
  {
    id: 'search-optimization',
    name: 'Smart Search',
    description: 'Intelligent search with context and filters',
    category: 'search',
    template: 'Search for {query} in Hacker News with {searchType} matching. Apply {contextFilters} and rank results by {rankingCriteria}.',
    parameters: [
      {
        name: 'query',
        label: 'Search Query',
        type: 'text',
        required: true,
        placeholder: 'Enter your search terms',
      },
      {
        name: 'searchType',
        label: 'Search Type',
        type: 'select',
        required: true,
        options: ['exact match', 'semantic search', 'fuzzy matching', 'conceptual search'],
        defaultValue: 'semantic search',
      },
      {
        name: 'contextFilters',
        label: 'Context Filters',
        type: 'multiselect',
        required: false,
        options: ['technical content', 'business discussions', 'news articles', 'personal experiences', 'tutorials'],
        defaultValue: ['technical content'],
      },
      {
        name: 'rankingCriteria',
        label: 'Ranking Criteria',
        type: 'select',
        required: true,
        options: ['relevance', 'recency', 'popularity', 'discussion quality', 'expert involvement'],
        defaultValue: 'relevance',
      },
    ],
    examples: [
      'Search for "kubernetes deployment" in Hacker News with semantic search matching. Apply technical content and rank results by relevance.',
    ],
    tags: ['search', 'smart', 'filtering'],
    difficulty: 'beginner',
    estimatedTime: '1-2 minutes',
  },
  {
    id: 'custom-query',
    name: 'Custom Query',
    description: 'Build your own custom query from scratch',
    category: 'custom',
    template: '{customQuery}',
    parameters: [
      {
        name: 'customQuery',
        label: 'Custom Query',
        type: 'text',
        required: true,
        placeholder: 'Write your custom LLM query here...',
        validation: {
          min: 10,
          message: 'Query must be at least 10 characters long',
        },
      },
    ],
    examples: [
      'Find all discussions about AI safety where users express concerns about AGI development.',
      'Compare the reception of different programming languages in job postings shared on HN.',
    ],
    tags: ['custom', 'flexible', 'advanced'],
    difficulty: 'advanced',
    estimatedTime: 'Variable',
  },
];

// Template categories for organization
export const TEMPLATE_CATEGORIES = [
  { value: 'analysis', label: 'Analysis', description: 'Deep analysis and insights' },
  { value: 'search', label: 'Search', description: 'Smart search and discovery' },
  { value: 'comparison', label: 'Comparison', description: 'Compare entities or topics' },
  { value: 'trend', label: 'Trends', description: 'Trend analysis over time' },
  { value: 'entity', label: 'Entities', description: 'Entity-focused queries' },
  { value: 'relation', label: 'Relations', description: 'Relationship exploration' },
  { value: 'custom', label: 'Custom', description: 'Build your own query' },
] as const;