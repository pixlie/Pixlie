use super::*;
use crate::llm::LLMProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub steps: Vec<PlanStep>,
    pub estimated_duration: Option<u64>,
    pub complexity: QueryComplexity,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: u32,
    pub description: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub depends_on: Vec<u32>,
    pub can_run_parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryComplexity {
    Simple,      // Single tool call
    Moderate,    // 2-3 sequential tool calls
    Complex,     // Multiple tools with dependencies
    VeryComplex, // Requires iteration and branching
}

#[allow(dead_code)]
pub struct QueryPlanner {
    llm_provider: Box<dyn LLMProvider>,
}

#[allow(dead_code)]
impl QueryPlanner {
    pub fn new(llm_provider: Box<dyn LLMProvider>) -> Self {
        Self { llm_provider }
    }

    pub async fn analyze_query(
        &self,
        query: &str,
        available_tools: &[ToolDescriptor],
    ) -> Result<QueryPlan, ConversationError> {
        let analysis_prompt = self.build_analysis_prompt(query, available_tools).await?;

        let llm_response = self
            .llm_provider
            .generate_response(&analysis_prompt, available_tools)
            .await
            .map_err(|e| ConversationError::LLMProviderError(e.to_string()))?;

        self.parse_plan_from_response(&llm_response, available_tools)
            .await
    }

    pub async fn optimize_plan(
        &self,
        plan: &QueryPlan,
        execution_context: &ConversationContext,
    ) -> Result<QueryPlan, ConversationError> {
        // Analyze previous execution history to optimize
        let optimization_suggestions = self
            .analyze_execution_history(&execution_context.execution_history)
            .await?;

        let mut optimized_plan = plan.clone();

        // Apply optimization suggestions
        for suggestion in optimization_suggestions {
            match suggestion {
                OptimizationSuggestion::ParallelizeSteps(step_ids) => {
                    self.parallelize_steps(&mut optimized_plan, step_ids)?;
                }
                OptimizationSuggestion::CacheResult(step_id, cache_key) => {
                    self.add_caching(&mut optimized_plan, step_id, cache_key)?;
                }
                OptimizationSuggestion::SkipRedundantStep(step_id) => {
                    self.remove_step(&mut optimized_plan, step_id)?;
                }
            }
        }

        Ok(optimized_plan)
    }

    pub fn validate_plan(
        &self,
        plan: &QueryPlan,
        available_tools: &[ToolDescriptor],
    ) -> Result<(), ConversationError> {
        // Check that all required tools are available
        for required_tool in &plan.required_tools {
            if !available_tools
                .iter()
                .any(|tool| tool.name == *required_tool)
            {
                return Err(ConversationError::PlanningFailed(format!(
                    "Required tool '{required_tool}' is not available"
                )));
            }
        }

        // Check for circular dependencies
        self.check_circular_dependencies(plan)?;

        // Validate step parameters
        for step in &plan.steps {
            self.validate_step_parameters(step, available_tools)?;
        }

        Ok(())
    }

    pub fn estimate_execution_time(
        &self,
        plan: &QueryPlan,
        execution_context: &ConversationContext,
    ) -> u64 {
        let mut total_time = 0;

        for step in &plan.steps {
            let estimated_step_time = self.estimate_step_time(step, execution_context);
            total_time += estimated_step_time;
        }

        // Adjust for parallelization
        let parallel_groups = self.identify_parallel_groups(plan);
        if parallel_groups.len() > 1 {
            // Reduce total time for parallel execution
            total_time = (total_time as f64 * 0.7) as u64;
        }

        total_time
    }

    #[allow(clippy::uninlined_format_args)]
    async fn build_analysis_prompt(
        &self,
        query: &str,
        available_tools: &[ToolDescriptor],
    ) -> Result<String, ConversationError> {
        let tools_description = available_tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"Analyze the following user query and create a step-by-step execution plan using the available tools.

User Query: "{query}"

Available Tools:
{tools_description}

Please provide a structured plan that includes:
1. Query complexity assessment (Simple/Moderate/Complex/VeryComplex)
2. List of required tools
3. Detailed execution steps with:
   - Step description
   - Tool to use
   - Parameters needed
   - Dependencies on other steps
   - Whether step can run in parallel

Consider data dependencies, tool capabilities, and execution efficiency.

Return the plan in JSON format with the following structure:
{{
  "complexity": "Simple|Moderate|Complex|VeryComplex",
  "required_tools": ["tool1", "tool2"],
  "steps": [
    {{
      "step_id": 1,
      "description": "Step description",
      "tool_name": "tool_name",
      "parameters": {{}},
      "depends_on": [],
      "can_run_parallel": false
    }}
  ]
}}"#
        ))
    }

    async fn parse_plan_from_response(
        &self,
        response: &str,
        _available_tools: &[ToolDescriptor],
    ) -> Result<QueryPlan, ConversationError> {
        // Try to extract JSON from the response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            ConversationError::PlanningFailed(format!("Failed to parse plan JSON: {e}"))
        })?;

        let complexity_str = parsed
            .get("complexity")
            .and_then(|v| v.as_str())
            .unwrap_or("Moderate");

        let complexity = match complexity_str {
            "Simple" => QueryComplexity::Simple,
            "Moderate" => QueryComplexity::Moderate,
            "Complex" => QueryComplexity::Complex,
            "VeryComplex" => QueryComplexity::VeryComplex,
            _ => QueryComplexity::Moderate,
        };

        let required_tools = parsed
            .get("required_tools")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let steps_array = parsed
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ConversationError::PlanningFailed("No steps found in plan".to_string())
            })?;

        let mut steps = Vec::new();
        for step_value in steps_array {
            let step_id = step_value
                .get("step_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(steps.len() as u64 + 1) as u32;

            let description = step_value
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown step")
                .to_string();

            let tool_name = step_value
                .get("tool_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_tool")
                .to_string();

            let parameters = step_value
                .get("parameters")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let depends_on = step_value
                .get("depends_on")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64())
                        .map(|n| n as u32)
                        .collect()
                })
                .unwrap_or_default();

            let can_run_parallel = step_value
                .get("can_run_parallel")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            steps.push(PlanStep {
                step_id,
                description,
                tool_name,
                parameters,
                depends_on,
                can_run_parallel,
            });
        }

        let estimated_duration = self.calculate_estimated_duration(&steps, &complexity);

        Ok(QueryPlan {
            steps,
            estimated_duration: Some(estimated_duration),
            complexity,
            required_tools,
        })
    }

    fn calculate_estimated_duration(
        &self,
        steps: &[PlanStep],
        complexity: &QueryComplexity,
    ) -> u64 {
        let base_time_per_step = match complexity {
            QueryComplexity::Simple => 2000,      // 2 seconds
            QueryComplexity::Moderate => 3000,    // 3 seconds
            QueryComplexity::Complex => 5000,     // 5 seconds
            QueryComplexity::VeryComplex => 8000, // 8 seconds
        };

        let total_steps = steps.len() as u64;
        let parallel_factor = if steps.iter().any(|s| s.can_run_parallel) {
            0.7
        } else {
            1.0
        };

        ((total_steps * base_time_per_step as u64) as f64 * parallel_factor) as u64
    }

    #[allow(clippy::uninlined_format_args)]
    async fn analyze_execution_history(
        &self,
        history: &[ToolExecution],
    ) -> Result<Vec<OptimizationSuggestion>, ConversationError> {
        let mut suggestions = Vec::new();

        // Analyze for patterns that suggest optimization opportunities

        // Check for redundant tool calls
        let mut tool_call_counts = std::collections::HashMap::new();
        for execution in history {
            *tool_call_counts.entry(&execution.tool_name).or_insert(0) += 1;
        }

        // Suggest caching for frequently used tools
        for (tool_name, count) in tool_call_counts {
            if count > 2 {
                suggestions.push(OptimizationSuggestion::CacheResult(
                    0,
                    format!("cache_{}", tool_name),
                ));
            }
        }

        // Analyze execution times to identify parallelization opportunities
        let slow_operations: Vec<_> = history
            .iter()
            .filter(|e| e.execution_time_ms.unwrap_or(0) > 5000)
            .collect();

        if slow_operations.len() > 1 {
            let step_ids: Vec<u32> = (1..=slow_operations.len() as u32).collect();
            suggestions.push(OptimizationSuggestion::ParallelizeSteps(step_ids));
        }

        Ok(suggestions)
    }

    fn parallelize_steps(
        &self,
        plan: &mut QueryPlan,
        step_ids: Vec<u32>,
    ) -> Result<(), ConversationError> {
        for step_id in step_ids {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
                step.can_run_parallel = true;
            }
        }
        Ok(())
    }

    fn add_caching(
        &self,
        plan: &mut QueryPlan,
        step_id: u32,
        _cache_key: String,
    ) -> Result<(), ConversationError> {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
            if let Some(params) = step.parameters.as_object_mut() {
                params.insert("use_cache".to_string(), serde_json::Value::Bool(true));
            }
        }
        Ok(())
    }

    fn remove_step(&self, plan: &mut QueryPlan, step_id: u32) -> Result<(), ConversationError> {
        plan.steps.retain(|s| s.step_id != step_id);
        // Update dependencies
        for step in &mut plan.steps {
            step.depends_on.retain(|&id| id != step_id);
        }
        Ok(())
    }

    #[allow(clippy::uninlined_format_args)]
    fn check_circular_dependencies(&self, plan: &QueryPlan) -> Result<(), ConversationError> {
        for step in &plan.steps {
            let mut visited = std::collections::HashSet::new();
            if self.has_circular_dependency(step.step_id, &plan.steps, &mut visited) {
                return Err(ConversationError::PlanningFailed(format!(
                    "Circular dependency detected involving step {}",
                    step.step_id
                )));
            }
        }
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn has_circular_dependency(
        &self,
        step_id: u32,
        steps: &[PlanStep],
        visited: &mut std::collections::HashSet<u32>,
    ) -> bool {
        if visited.contains(&step_id) {
            return true;
        }

        visited.insert(step_id);

        if let Some(step) = steps.iter().find(|s| s.step_id == step_id) {
            for &dep_id in &step.depends_on {
                if self.has_circular_dependency(dep_id, steps, visited) {
                    return true;
                }
            }
        }

        visited.remove(&step_id);
        false
    }

    #[allow(clippy::uninlined_format_args)]
    fn validate_step_parameters(
        &self,
        step: &PlanStep,
        available_tools: &[ToolDescriptor],
    ) -> Result<(), ConversationError> {
        let tool = available_tools
            .iter()
            .find(|t| t.name == step.tool_name)
            .ok_or_else(|| {
                ConversationError::PlanningFailed(format!("Tool '{}' not found", step.tool_name))
            })?;

        // Basic validation - in a real implementation, this would be more thorough
        if step.parameters.is_null() && !tool.parameters.is_null() {
            return Err(ConversationError::PlanningFailed(format!(
                "Step {} missing required parameters",
                step.step_id
            )));
        }

        Ok(())
    }

    fn estimate_step_time(&self, step: &PlanStep, _context: &ConversationContext) -> u64 {
        // Basic estimation based on tool type
        match step.tool_name.as_str() {
            name if name.contains("search") => 3000,
            name if name.contains("analyze") => 5000,
            name if name.contains("query") => 2000,
            _ => 3000,
        }
    }

    fn identify_parallel_groups(&self, plan: &QueryPlan) -> Vec<Vec<u32>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for step in &plan.steps {
            if step.can_run_parallel {
                current_group.push(step.step_id);
            } else {
                if !current_group.is_empty() {
                    groups.push(current_group.clone());
                    current_group.clear();
                }
                groups.push(vec![step.step_id]);
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum OptimizationSuggestion {
    ParallelizeSteps(Vec<u32>),
    CacheResult(u32, String),
    #[allow(dead_code)]
    SkipRedundantStep(u32),
}
