//! # Alias Editor Plugin
//!
//! This plugin provides a visual block-based editor for creating type aliases.
//! It supports .alias files (folder-based) that contain visual type compositions.
//!
//! ## File Types
//!
//! - **Type Alias** (.alias folder)
//!   - Contains `alias.json` with the type alias definition
//!   - Appears as a single file in the file drawer
//!
//! ## Editors
//!
//! - **Alias Editor**: Visual block-based type composition interface

use plugin_editor_api::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// Alias Editor modules
pub mod type_block;
pub mod constructor_palette;
pub mod block_canvas;
pub mod visual_editor;
pub mod type_palette;

// Re-export main types
pub use visual_editor::{VisualAliasEditor as AliasEditor, ShowTypePickerRequest};
pub use type_block::{TypeBlock, BlockId};
pub use constructor_palette::{ConstructorPalette, TypeSelected};
pub use block_canvas::{BlockCanvas, DragState, DropTarget};
pub use type_palette::{TypeLibraryPalette, TypeItem};

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<AliasEditorWrapper>,
}

/// The Alias Editor Plugin
pub struct AliasEditorPlugin {
    /// CRITICAL: Plugin owns ALL editor instances to prevent memory leaks!
    /// The main app only gets raw pointers - it NEVER owns the Arc or Box.
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for AliasEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for AliasEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.alias-editor"),
            name: "Alias Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Visual block-based editor for creating type aliases".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![
            FileTypeDefinition {
                id: FileTypeId::new("alias"),
                extension: "alias".to_string(),
                display_name: "Type Alias".to_string(),
                icon: ui::IconName::Code,
                color: gpui::rgb(0x3F51B5).into(),
                structure: FileStructure::FolderBased {
                    marker_file: "alias.json".to_string(),
                    template_structure: vec![],
                },
                default_content: json!({
                    "name": "NewAlias",
                    "target": "i32"
                }),
                categories: vec!["Types".to_string()],
            }
        ]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("alias-editor"),
            display_name: "Alias Editor".into(),
            supported_file_types: vec![FileTypeId::new("alias")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<(Arc<dyn PanelView>, Box<dyn EditorInstance>), PluginError> {

        logger.info("ALIAS EDITOR LOADED!!");

        logger.info(&format!("Creating editor with ID: {}", editor_id.as_str()));
        if editor_id.as_str() == "alias-editor" {
            let actual_path = if file_path.is_dir() {
                file_path.join("alias.json")
            } else {
                file_path.clone()
            };

            // Create a view context for the panel
            let panel = cx.new(|cx| {
                AliasEditor::new_with_file(actual_path.clone(), window, cx)
            });

            // Wrap the panel in Arc - will be shared with main app
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());

            // Clone file_path for logging
            let file_path_for_log = file_path.clone();

            // Create the wrapper for EditorInstance
            let wrapper = Box::new(AliasEditorWrapper {
                panel: panel.into(),
                file_path,
            });

            // Generate unique ID for this editor
            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            // CRITICAL: Store Arc and Box in plugin's HashMap to keep them alive!
            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created alias editor instance {} for {:?}", id, file_path_for_log);

            // Return Arc (main app will clone it) and Box for EditorInstance
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    fn on_load(&mut self) {
        log::info!("Alias Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        // Clear all editors when plugin unloads
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("Alias Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

/// Wrapper to bridge Entity<AliasEditor> to EditorInstance trait
#[derive(Clone)]
pub struct AliasEditorWrapper {
    panel: Entity<AliasEditor>,
    file_path: std::path::PathBuf,
}

impl plugin_editor_api::EditorInstance for AliasEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_save(window, cx)
        })
    }

    fn reload(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_reload(window, cx)
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Export the plugin using the provided macro
export_plugin!(AliasEditorPlugin);
