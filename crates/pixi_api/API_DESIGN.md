# Pixi API Design Document

## Overview

This document outlines the design for a unified `pixi_api` crate that serves as a high-level abstraction layer for Pixi operations. The goal is to provide a single, consistent API that can be consumed by:

- **pixi_cli**: The command-line interface
- **pixi_gui**: The Tauri-based graphical user interface  
- **Integration tests**: Rust integration tests
- **Future consumers**: Any other Rust applications or services

## Goals

- **Single Source of Truth**: Eliminate code duplication between CLI, GUI, and integration tests
- **Clean Separation**: Separate business logic from presentation concerns (CLI output, user interaction)
- **Serializable**: All types must be serializable for Tauri FFI compatibility

## Architecture

```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   pixi_gui      │  │  Integration    │  │   pixi_cli      │
│   (Tauri)       │  │     Tests       │  │                 │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                     │                     │
         └─────────────────────┼─────────────────────┘
                               │
                    ┌─────────────────┐
                    │   pixi_api      │ ← New abstraction layer
                    │                 │
                    └─────────────────┘
                               │
                    ┌─────────────────┐
                    │   pixi_core     │ ← Existing business logic
                    │                 │
                    └─────────────────┘
```

## Core Design Principles

### 1. Shared Structs with Conditional Features

Instead of duplicating structs between CLI and API, we use a single struct with conditional compilation:

```rust
// pixi_api/src/init.rs
#[cfg_attr(feature = "cli", derive(clap::Parser))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InitOptions {
    #[cfg_attr(feature = "cli", arg(default_value = "."))]
    pub path: PathBuf,
    
    #[cfg_attr(feature = "cli", arg(short, long = "channel"))]
    pub channels: Option<Vec<NamedChannelOrUrl>>,
    
    // ... other fields
}
```

**Benefits:**
- No code duplication
- Single source of truth for all option definitions
- Clap annotations only included when needed
- Automatically serializable for Tauri

### 2. Business Logic Separation

**Before (problematic):**
```rust
// Mixed business logic with CLI presentation
pub async fn execute(args: Args) -> miette::Result<()> {
    let config = Config::load_global();
    let workspace = create_workspace(&args)?;
    
    // CLI-specific output mixed in
    eprintln!("✔ Created {}", workspace.path.display());
    
    // CLI-specific user interaction
    let response = dialoguer::Confirm::new()
        .with_prompt("Extend pyproject.toml?")
        .interact()?;
}
```

**After (clean separation):**
```rust
// pixi_api - Pure business logic
pub async fn init_workspace(options: InitOptions) -> Result<InitResult, PixiApiError> {
    let config = Config::load_global();
    
    if requires_user_input(&options) {
        return Ok(InitResult {
            required_user_input: Some(UserInputRequired::ExtendPyproject { 
                question: "Extend pyproject.toml?".to_string() 
            }),
            ..Default::default()
        });
    }
    
    let workspace = create_workspace(&options, &config)?;
    Ok(InitResult {
        created_files: workspace.created_files,
        workspace_name: workspace.name,
        required_user_input: None,
    })
}

// pixi_cli - Presentation layer
pub async fn execute(args: Args) -> miette::Result<()> {
    let api = PixiApi::new()?;
    let result = api.init_workspace(args.into()).await?;
    
    match result.required_user_input {
        Some(UserInputRequired::ExtendPyproject { question }) => {
            let response = dialoguer::Confirm::new()
                .with_prompt(&question)
                .interact()?;
            
            let final_result = api.init_workspace_with_response(args.into(), response).await?;
            print_success(&final_result);
        },
        None => print_success(&result),
    }
}
```

### 3. Explicit User Interaction Modeling

Instead of hiding user interaction within business logic, we model it explicitly:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum UserInputRequired {
    ExtendExistingPyproject {
        existing_path: PathBuf,
        question: String,
        default: bool,
    },
    ConfirmOverwrite {
        file_path: PathBuf,
        question: String,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InitResult {
    pub created_files: Vec<PathBuf>,
    pub workspace_name: String,
    pub environments: Vec<String>,
    pub required_user_input: Option<UserInputRequired>,
}
```

**Benefits:**
- GUI can implement custom dialogs
- CLI can use terminal prompts
- Tests can provide programmatic responses
- Business logic remains pure and testable

## API Structure

### Core API Entry Point

```rust
pub struct PixiApi {
    config: pixi_config::Config,
}

impl PixiApi {
    pub fn new() -> Result<Self, PixiApiError>;
    pub fn with_config(config: pixi_config::Config) -> Self;
    
    // Project lifecycle
    pub async fn init_project(&self, options: InitOptions) -> Result<InitResult, PixiApiError>;
    pub fn open_project(&self, path: impl AsRef<Path>) -> Result<PixiProject, PixiApiError>;
}
```

### Project Management

```rust
pub struct PixiProject {
    workspace: pixi_core::Workspace,
    config: pixi_config::Config,
}

impl PixiProject {
    // Dependency management
    pub async fn add_dependencies(&mut self, options: AddDependencyOptions) -> Result<AddResult, PixiApiError>;
    pub async fn remove_dependencies(&mut self, options: RemoveOptions) -> Result<RemoveResult, PixiApiError>;
    
    // Environment management  
    pub async fn install(&self, options: InstallOptions) -> Result<InstallResult, PixiApiError>;
    pub async fn lock(&mut self, options: LockOptions) -> Result<LockResult, PixiApiError>;
    
    // Task management
    pub async fn add_task(&mut self, task: TaskDefinition) -> Result<(), PixiApiError>;
    pub async fn run_task(&self, name: &str, options: RunOptions) -> Result<TaskResult, PixiApiError>;
    
    // Information queries
    pub fn get_dependencies(&self) -> Result<Vec<DependencyInfo>, PixiApiError>;
    pub fn get_environments(&self) -> Result<Vec<EnvironmentInfo>, PixiApiError>;
    pub fn get_tasks(&self) -> Result<Vec<TaskInfo>, PixiApiError>;
}
```

### Type-Safe Options

```rust
#[cfg_attr(feature = "cli", derive(clap::Parser))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddDependencyOptions {
    #[cfg_attr(feature = "cli", arg(help = "Package specifications to add"))]
    pub specs: Vec<String>,
    
    #[cfg_attr(feature = "cli", arg(long, help = "Add as PyPI dependency"))]
    pub pypi: bool,
    
    #[cfg_attr(feature = "cli", arg(short, long, help = "Target platforms"))]
    pub platforms: Vec<Platform>,
    
    #[cfg_attr(feature = "cli", arg(long, help = "Don't run install after adding"))]
    pub no_install: bool,
}
```

## Consumer Integration

### CLI Integration

```rust
// pixi_cli uses the API with minimal wrapper
pub use pixi_api::init::InitOptions as Args;

pub async fn execute(args: Args) -> miette::Result<()> {
    let api = PixiApi::new().into_diagnostic()?;
    let result = api.init_project(args).await.into_diagnostic()?;
    
    // Handle CLI-specific presentation
    handle_result(result)?;
    Ok(())
}
```

### Tauri Commands

```rust
#[tauri::command]
pub async fn init_project(options: InitOptions) -> Result<InitResult, String> {
    let api = PixiApi::new().map_err(|e| e.to_string())?;
    api.init_project(options).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project_info(path: String) -> Result<ProjectInfo, String> {
    let api = PixiApi::new().map_err(|e| e.to_string())?;
    let project = api.open_project(path).map_err(|e| e.to_string())?;
    Ok(project.get_info())
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_init_with_channels() {
    let temp_dir = tempdir().unwrap();
    let options = InitOptions {
        path: temp_dir.path().to_path_buf(),
        channels: Some(vec!["conda-forge".parse().unwrap()]),
        ..Default::default()
    };
    
    let api = PixiApi::new().unwrap();
    let result = api.init_project(options).await.unwrap();
    
    assert_eq!(result.workspace_name, "test_workspace");
    assert!(result.created_files.contains(&temp_dir.path().join("pixi.toml")));
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum PixiApiError {
    #[error("Workspace error: {0}")]
    Workspace(#[from] pixi_core::WorkspaceError),
    
    #[error("Config error: {0}")]  
    Config(#[from] pixi_config::ConfigError),
    
    #[error("Operation failed: {message}")]
    Generic { message: String },
}

// Automatic conversion for Tauri
impl From<PixiApiError> for String {
    fn from(err: PixiApiError) -> String {
        err.to_string()
    }
}
```

## Progress Reporting

For long-running operations, we provide an event system:

```rust
pub trait ProgressHandler: Send + Sync {
    fn on_progress(&self, event: ProgressEvent);
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ProgressEvent {
    Starting { operation: String },
    Progress { current: u64, total: u64, message: String },
    Finished { operation: String },
    Error { error: String },
}

impl PixiProject {
    pub fn with_progress_handler(mut self, handler: Box<dyn ProgressHandler>) -> Self;
}
```

## Migration Strategy

1. **Phase 1**: Create basic `pixi_api` structure with `init` command
2. **Phase 2**: Migrate integration tests to use new API
3. **Phase 3**: Update CLI to use shared structs and API
4. **Phase 4**: Add remaining commands (`add`, `install`, `run`, etc.)
5. **Phase 5**: GUI can consume the mature API

## Benefits

### For CLI
- Reduced code duplication
- Cleaner separation of concerns
- Same type definitions as API
- Gradual migration possible

### For GUI (Tauri)
- All types automatically serializable
- Async API perfect for reactive UIs
- Progress events for user feedback
- Type-safe API calls

### For Integration Tests
- Replace builder pattern with type-safe options
- No CLI dependencies needed
- Easy to mock and test
- Less boilerplate code