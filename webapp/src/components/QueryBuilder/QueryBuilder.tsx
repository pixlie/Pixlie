import { useState, useCallback, useEffect, useMemo } from 'react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { 
  Sparkles, 
  Copy, 
  RotateCcw, 
  BookOpen, 
  ChevronDown,
  ChevronUp,
  Play,
  Save
} from 'lucide-react';
import { cn } from '../../lib/utils';
import { QueryParameterInput } from './QueryParameterInput';
import { 
  DEFAULT_QUERY_TEMPLATES, 
  TEMPLATE_CATEGORIES,
  type QueryTemplate
} from './QueryTemplate';

interface QueryBuilderProps {
  onSubmit?: (query: string, metadata?: { template?: string; parameters?: Record<string, unknown> }) => void;
  onSave?: (query: string, name: string) => void;
  defaultTemplate?: string;
  className?: string;
}

interface QueryBuilderState {
  selectedTemplate: QueryTemplate | null;
  parameters: Record<string, unknown>;
  customQuery: string;
  generatedQuery: string;
  errors: Record<string, string>;
  savedQueries: Array<{ name: string; query: string; timestamp: string }>;
}

const API_BASE_URL = 'http://localhost:8080/api';

export function QueryBuilder({ 
  onSubmit, 
  onSave, 
  defaultTemplate, 
  className 
}: QueryBuilderProps) {
  const [state, setState] = useState<QueryBuilderState>({
    selectedTemplate: null,
    parameters: {},
    customQuery: '',
    generatedQuery: '',
    errors: {},
    savedQueries: [],
  });

  const [isExpanded, setIsExpanded] = useState(true);
  const [activeCategory, setActiveCategory] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [showSavedQueries, setShowSavedQueries] = useState(false);

  // Filter templates based on category and search
  const filteredTemplates = useMemo(() => {
    let templates = DEFAULT_QUERY_TEMPLATES;
    
    if (activeCategory !== 'all') {
      templates = templates.filter(t => t.category === activeCategory);
    }
    
    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      templates = templates.filter(t => 
        t.name.toLowerCase().includes(term) ||
        t.description.toLowerCase().includes(term) ||
        t.tags.some(tag => tag.toLowerCase().includes(term))
      );
    }
    
    return templates;
  }, [activeCategory, searchTerm]);

  // Load saved queries from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('pixlie-saved-queries');
    if (saved) {
      try {
        const queries = JSON.parse(saved);
        setState(prev => ({ ...prev, savedQueries: queries }));
      } catch (error) {
        console.error('Failed to load saved queries:', error);
      }
    }
  }, []);

  // Entity search function
  const handleEntitySearch = useCallback(async (query: string): Promise<string[]> => {
    try {
      const response = await fetch(`${API_BASE_URL}/entities/search?q=${encodeURIComponent(query)}&limit=10`);
      if (response.ok) {
        const data = await response.json();
        return data.entities.map((e: { entity: { entity_value: string } }) => e.entity.entity_value);
      }
    } catch (error) {
      console.error('Entity search failed:', error);
    }
    return [];
  }, []);

  const generateQuery = useCallback((template: QueryTemplate, parameters: Record<string, unknown>) => {
    let query = template.template;
    
    // Replace parameters in template
    template.parameters.forEach(param => {
      const value = parameters[param.name];
      let replacementValue = '';
      
      if (value !== undefined && value !== null && value !== '') {
        if (Array.isArray(value)) {
          replacementValue = value.join(', ');
        } else if (typeof value === 'object' && value !== null) {
          // Handle date range
          const dateRange = value as { start?: string; end?: string };
          if (dateRange.start && dateRange.end) {
            replacementValue = `${dateRange.start} to ${dateRange.end}`;
          } else if (dateRange.start) {
            replacementValue = `from ${dateRange.start}`;
          } else if (dateRange.end) {
            replacementValue = `until ${dateRange.end}`;
          }
        } else {
          replacementValue = String(value);
        }
      }
      
      query = query.replace(new RegExp(`{${param.name}}`, 'g'), replacementValue);
    });
    
    setState(prev => ({ ...prev, generatedQuery: query }));
  }, []);

  const handleTemplateSelect = useCallback((template: QueryTemplate) => {
    setState(prev => ({
      ...prev,
      selectedTemplate: template,
      parameters: template.parameters.reduce((acc, param) => {
        acc[param.name] = param.defaultValue || '';
        return acc;
      }, {} as Record<string, unknown>),
      errors: {},
    }));
    generateQuery(template, {});
  }, [generateQuery]);

  const handleParameterChange = useCallback((paramName: string, value: unknown) => {
    setState(prev => {
      const newParameters = { ...prev.parameters, [paramName]: value };
      const newErrors = { ...prev.errors };
      delete newErrors[paramName]; // Clear error when user changes value
      
      if (prev.selectedTemplate) {
        generateQuery(prev.selectedTemplate, newParameters);
      }
      
      return {
        ...prev,
        parameters: newParameters,
        errors: newErrors,
      };
    });
  }, [generateQuery]);

  // Set default template
  useEffect(() => {
    if (defaultTemplate) {
      const template = DEFAULT_QUERY_TEMPLATES.find(t => t.id === defaultTemplate);
      if (template) {
        setState(prev => ({
          ...prev,
          selectedTemplate: template,
          parameters: template.parameters.reduce((acc, param) => {
            acc[param.name] = param.defaultValue || '';
            return acc;
          }, {} as Record<string, unknown>),
          errors: {},
        }));
        generateQuery(template, {});
      }
    }
  }, [defaultTemplate, generateQuery]);

  const validateParameters = useCallback(() => {
    if (!state.selectedTemplate) return true;
    
    const errors: Record<string, string> = {};
    
    state.selectedTemplate.parameters.forEach(param => {
      const value = state.parameters[param.name];
      
      if (param.required && (value === undefined || value === null || value === '')) {
        errors[param.name] = `${param.label} is required`;
        return;
      }
      
      if (param.validation && value) {
        const validation = param.validation;
        
        if (validation.min !== undefined && typeof value === 'number' && value < validation.min) {
          errors[param.name] = `Minimum value is ${validation.min}`;
        }
        
        if (validation.max !== undefined && typeof value === 'number' && value > validation.max) {
          errors[param.name] = `Maximum value is ${validation.max}`;
        }
        
        if (validation.pattern && typeof value === 'string') {
          const regex = new RegExp(validation.pattern);
          if (!regex.test(value)) {
            errors[param.name] = validation.message || 'Invalid format';
          }
        }
      }
    });
    
    setState(prev => ({ ...prev, errors }));
    return Object.keys(errors).length === 0;
  }, [state.selectedTemplate, state.parameters]);

  const handleSubmit = useCallback(() => {
    const query = state.selectedTemplate ? state.generatedQuery : state.customQuery;
    
    if (!query.trim()) {
      alert('Please enter a query or select a template');
      return;
    }
    
    if (state.selectedTemplate && !validateParameters()) {
      return;
    }
    
    onSubmit?.(query, {
      template: state.selectedTemplate?.id,
      parameters: state.parameters,
    });
  }, [state, validateParameters, onSubmit]);

  const handleSaveQuery = useCallback(() => {
    const query = state.selectedTemplate ? state.generatedQuery : state.customQuery;
    const name = prompt('Enter a name for this query:');
    
    if (name && query.trim()) {
      const newQuery = {
        name,
        query,
        timestamp: new Date().toISOString(),
      };
      
      const updatedQueries = [...state.savedQueries, newQuery];
      setState(prev => ({ ...prev, savedQueries: updatedQueries }));
      localStorage.setItem('pixlie-saved-queries', JSON.stringify(updatedQueries));
      
      onSave?.(query, name);
    }
  }, [state, onSave]);

  const handleCopyQuery = useCallback(() => {
    const query = state.selectedTemplate ? state.generatedQuery : state.customQuery;
    navigator.clipboard.writeText(query);
  }, [state]);

  const handleReset = useCallback(() => {
    setState(prev => ({
      ...prev,
      selectedTemplate: null,
      parameters: {},
      customQuery: '',
      generatedQuery: '',
      errors: {},
    }));
  }, []);

  return (
    <Card className={cn('w-full', className)}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-blue-600" />
            <CardTitle>LLM Query Builder</CardTitle>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </Button>
        </div>
      </CardHeader>

      {isExpanded && (
        <CardContent className="space-y-6">
          {/* Template Search and Categories */}
          <div className="space-y-4">
            <div className="flex gap-2">
              <div className="flex-1">
                <Input
                  placeholder="Search templates..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full"
                />
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowSavedQueries(!showSavedQueries)}
              >
                <BookOpen className="w-4 h-4 mr-2" />
                Saved
              </Button>
            </div>

            {/* Category filters */}
            <div className="flex flex-wrap gap-2">
              <Button
                variant={activeCategory === 'all' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setActiveCategory('all')}
              >
                All
              </Button>
              {TEMPLATE_CATEGORIES.map(category => (
                <Button
                  key={category.value}
                  variant={activeCategory === category.value ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setActiveCategory(category.value)}
                  title={category.description}
                >
                  {category.label}
                </Button>
              ))}
            </div>
          </div>

          {/* Saved Queries */}
          {showSavedQueries && state.savedQueries.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Saved Queries</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 max-h-32 overflow-y-auto">
                  {state.savedQueries.map((saved, index) => (
                    <button
                      key={index}
                      className="w-full text-left p-2 hover:bg-gray-50 rounded border"
                      onClick={() => setState(prev => ({ ...prev, customQuery: saved.query }))}
                    >
                      <div className="font-medium text-sm">{saved.name}</div>
                      <div className="text-xs text-gray-500 truncate">{saved.query}</div>
                    </button>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Template Selection */}
          <div className="space-y-3">
            <h3 className="text-sm font-medium">Choose a Template</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-64 overflow-y-auto">
              {filteredTemplates.map(template => (
                <Card
                  key={template.id}
                  className={cn(
                    'cursor-pointer transition-colors hover:bg-gray-50',
                    state.selectedTemplate?.id === template.id && 'ring-2 ring-blue-500'
                  )}
                  onClick={() => handleTemplateSelect(template)}
                >
                  <CardContent className="p-4">
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <h4 className="font-medium text-sm">{template.name}</h4>
                        <span className={cn(
                          'px-2 py-1 text-xs rounded',
                          template.difficulty === 'beginner' && 'bg-green-100 text-green-700',
                          template.difficulty === 'intermediate' && 'bg-yellow-100 text-yellow-700',
                          template.difficulty === 'advanced' && 'bg-red-100 text-red-700'
                        )}>
                          {template.difficulty}
                        </span>
                      </div>
                      <p className="text-xs text-gray-600">{template.description}</p>
                      <div className="flex flex-wrap gap-1">
                        {template.tags.slice(0, 3).map(tag => (
                          <span key={tag} className="px-1 py-0.5 bg-gray-100 text-gray-600 text-xs rounded">
                            {tag}
                          </span>
                        ))}
                      </div>
                      {template.estimatedTime && (
                        <div className="text-xs text-gray-500">⏱️ {template.estimatedTime}</div>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>

          {/* Parameter Inputs */}
          {state.selectedTemplate && (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Configure Parameters</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {state.selectedTemplate.parameters.map(parameter => (
                  <QueryParameterInput
                    key={parameter.name}
                    parameter={parameter}
                    value={state.parameters[parameter.name]}
                    onChange={(value) => handleParameterChange(parameter.name, value)}
                    onEntitySearch={parameter.type === 'entity' ? handleEntitySearch : undefined}
                    error={state.errors[parameter.name]}
                  />
                ))}
              </CardContent>
            </Card>
          )}

          {/* Custom Query Input */}
          {!state.selectedTemplate && (
            <div className="space-y-2">
              <label className="block text-sm font-medium">Custom Query</label>
              <textarea
                value={state.customQuery}
                onChange={(e) => setState(prev => ({ ...prev, customQuery: e.target.value }))}
                placeholder="Enter your custom LLM query here..."
                className="w-full min-h-32 p-3 border border-gray-300 rounded-md resize-y"
              />
            </div>
          )}

          {/* Generated Query Preview */}
          {(state.generatedQuery || state.customQuery) && (
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-sm">Generated Query</CardTitle>
                  <div className="flex gap-2">
                    <Button variant="ghost" size="sm" onClick={handleCopyQuery}>
                      <Copy className="w-4 h-4" />
                    </Button>
                    <Button variant="ghost" size="sm" onClick={handleSaveQuery}>
                      <Save className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="p-3 bg-gray-50 rounded border font-mono text-sm">
                  {state.selectedTemplate ? state.generatedQuery : state.customQuery}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Action Buttons */}
          <div className="flex items-center justify-between pt-4 border-t">
            <Button variant="outline" onClick={handleReset}>
              <RotateCcw className="w-4 h-4 mr-2" />
              Reset
            </Button>
            
            <div className="flex gap-2">
              <Button variant="outline" onClick={handleCopyQuery}>
                <Copy className="w-4 h-4 mr-2" />
                Copy
              </Button>
              <Button 
                onClick={handleSubmit}
                disabled={!(state.generatedQuery || state.customQuery)}
              >
                <Play className="w-4 h-4 mr-2" />
                Execute Query
              </Button>
            </div>
          </div>
        </CardContent>
      )}
    </Card>
  );
}