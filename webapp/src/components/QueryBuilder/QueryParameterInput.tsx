import { useState, useCallback } from 'react';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { X, Search } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { QueryParameter } from './QueryTemplate';

interface QueryParameterInputProps {
  parameter: QueryParameter;
  value: unknown;
  onChange: (value: unknown) => void;
  onEntitySearch?: (query: string) => Promise<string[]>;
  error?: string;
}

export function QueryParameterInput({
  parameter,
  value,
  onChange,
  onEntitySearch,
  error,
}: QueryParameterInputProps) {
  const [entitySuggestions, setEntitySuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [isSearching, setIsSearching] = useState(false);

  const handleEntitySearch = useCallback(async (query: string) => {
    if (!onEntitySearch || query.length < 2) {
      setEntitySuggestions([]);
      setShowSuggestions(false);
      return;
    }

    setIsSearching(true);
    try {
      const suggestions = await onEntitySearch(query);
      setEntitySuggestions(suggestions);
      setShowSuggestions(suggestions.length > 0);
    } catch (error) {
      console.error('Entity search failed:', error);
      setEntitySuggestions([]);
      setShowSuggestions(false);
    } finally {
      setIsSearching(false);
    }
  }, [onEntitySearch]);

  const handleMultiselectToggle = useCallback((option: string) => {
    const currentValues = Array.isArray(value) ? value : [];
    const newValues = currentValues.includes(option)
      ? currentValues.filter(v => v !== option)
      : [...currentValues, option];
    onChange(newValues);
  }, [value, onChange]);

  const renderTextInput = () => (
    <Input
      type="text"
      value={String(value || '')}
      onChange={(e) => onChange(e.target.value)}
      placeholder={parameter.placeholder}
      className={cn(error && 'border-red-500')}
    />
  );

  const renderNumberInput = () => (
    <Input
      type="number"
      value={Number(value) || ''}
      onChange={(e) => onChange(Number(e.target.value))}
      placeholder={parameter.placeholder}
      min={parameter.validation?.min}
      max={parameter.validation?.max}
      className={cn(error && 'border-red-500')}
    />
  );

  const renderSelectInput = () => (
    <select
      value={String(value || '')}
      onChange={(e) => onChange(e.target.value)}
      className={cn(
        'w-full appearance-none bg-white border border-gray-300 rounded-md px-3 py-2 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent',
        error && 'border-red-500'
      )}
    >
      <option value="">Select {parameter.label.toLowerCase()}</option>
      {parameter.options?.map(option => (
        <option key={option} value={option}>
          {option.charAt(0).toUpperCase() + option.slice(1)}
        </option>
      ))}
    </select>
  );

  const renderMultiselectInput = () => {
    const selectedValues = Array.isArray(value) ? value : [];
    
    return (
      <div className="space-y-2">
        <div className="flex flex-wrap gap-2">
          {parameter.options?.map(option => (
            <Button
              key={option}
              type="button"
              variant={selectedValues.includes(option) ? 'default' : 'outline'}
              size="sm"
              onClick={() => handleMultiselectToggle(option)}
              className="text-xs"
            >
              {option.charAt(0).toUpperCase() + option.slice(1)}
              {selectedValues.includes(option) && (
                <X className="w-3 h-3 ml-1" />
              )}
            </Button>
          ))}
        </div>
        {selectedValues.length > 0 && (
          <div className="text-xs text-gray-600">
            Selected: {selectedValues.join(', ')}
          </div>
        )}
      </div>
    );
  };

  const renderEntityInput = () => (
    <div className="relative">
      <div className="relative">
        <Input
          type="text"
          value={String(value || '')}
          onChange={(e) => {
            onChange(e.target.value);
            handleEntitySearch(e.target.value);
          }}
          placeholder={parameter.placeholder}
          className={cn('pr-10', error && 'border-red-500')}
          onFocus={() => value && handleEntitySearch(String(value))}
          onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
        />
        <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
          {isSearching ? (
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500"></div>
          ) : (
            <Search className="w-4 h-4 text-gray-400" />
          )}
        </div>
      </div>
      
      {showSuggestions && entitySuggestions.length > 0 && (
        <Card className="absolute z-10 w-full mt-1 max-h-48 overflow-y-auto">
          <CardContent className="p-0">
            {entitySuggestions.map((suggestion, index) => (
              <button
                key={index}
                type="button"
                className="w-full text-left px-3 py-2 hover:bg-gray-50 border-b border-gray-100 last:border-b-0"
                onClick={() => {
                  onChange(suggestion);
                  setShowSuggestions(false);
                }}
              >
                {suggestion}
              </button>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );

  const renderDateRangeInput = () => {
    const dateRange = value as { start?: string; end?: string } || {};
    
    return (
      <div className="grid grid-cols-2 gap-2">
        <div>
          <label className="block text-xs text-gray-600 mb-1">Start Date</label>
          <Input
            type="date"
            value={dateRange.start || ''}
            onChange={(e) => onChange({ ...dateRange, start: e.target.value })}
            className={cn(error && 'border-red-500')}
          />
        </div>
        <div>
          <label className="block text-xs text-gray-600 mb-1">End Date</label>
          <Input
            type="date"
            value={dateRange.end || ''}
            onChange={(e) => onChange({ ...dateRange, end: e.target.value })}
            className={cn(error && 'border-red-500')}
          />
        </div>
      </div>
    );
  };

  const renderInput = () => {
    switch (parameter.type) {
      case 'text':
        return renderTextInput();
      case 'number':
        return renderNumberInput();
      case 'select':
        return renderSelectInput();
      case 'multiselect':
        return renderMultiselectInput();
      case 'entity':
        return renderEntityInput();
      case 'date-range':
        return renderDateRangeInput();
      default:
        return renderTextInput();
    }
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-1">
        <label className="block text-sm font-medium text-gray-700">
          {parameter.label}
        </label>
        {parameter.required && (
          <span className="text-red-500 text-xs">*</span>
        )}
      </div>
      
      {renderInput()}
      
      {error && (
        <div className="text-red-500 text-xs">
          {error}
        </div>
      )}
      
      {parameter.validation?.message && !error && (
        <div className="text-gray-500 text-xs">
          {parameter.validation.message}
        </div>
      )}
    </div>
  );
}