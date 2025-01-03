#![doc = include_str!("../readme.md")]

mod focus;

use ratatui::layout::Rect;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

pub use crate::focus::{handle_focus, Focus, FocusBuilder};

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::*;
}

/// Holds the flags for the focus.
///
/// Add this to the widget state.
///
/// This struct is intended to be cloned and uses a Rc internally
/// to share the state.
///
///
///
/// __Attention__
/// Equality for FocusFlag means pointer-equality of the underlying
/// Rc using Rc::ptr_eq.
///
/// __See__
/// [HasFocus], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
/// __See__
/// [FocusAdapter] to use with widgets that don't have
/// their own focus flag.
#[derive(Clone, Default)]
pub struct FocusFlag(Rc<FocusFlagCore>);

/// The same as FocusFlag, but distinct to mark the focus for
/// a container.
///
/// This serves the purpose of
///
/// * summarizing the focus for the container. If any of the
///     widgets of the container has the focus, the container
///     itself has the focus.
/// * identifying the container for some functions on Focus.
#[derive(Clone, Default)]
pub struct ContainerFlag(Rc<FocusFlagCore>);

/// Equality for FocusFlag means pointer equality of the underlying
/// Rc using Rc::ptr_eq.
impl PartialEq for FocusFlag {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for FocusFlag {}

impl Hash for FocusFlag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl Display for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "|{}|", self.0.name)
    }
}

impl HasFocus for FocusFlag {
    fn focus(&self) -> FocusFlag {
        self.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

/// Equality for ContainerFlag means pointer equality of the underlying
/// Rc using Rc::ptr_eq.
impl PartialEq for ContainerFlag {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ContainerFlag {}

impl Hash for ContainerFlag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl Display for ContainerFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "|{}|", self.0.name)
    }
}

impl FocusContainer for ContainerFlag {
    fn build(&self, _builder: &mut FocusBuilder) {
        // no widgets
    }

    fn container(&self) -> Option<ContainerFlag> {
        Some(self.clone())
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

// not Clone, always Rc<>
#[derive(Default)]
struct FocusFlagCore {
    /// Field name for debugging purposes.
    name: Box<str>,
    /// Focus.
    focus: Cell<bool>,
    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    gained: Cell<bool>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    lost: Cell<bool>,
}

/// Focus navigation for widgets.
///
/// The effects that hinder focus-change (`Reach*`, `Lock`) only work
/// when navigation changes via next()/prev()/focus_at().
/// Programmatic focus changes are always possible.
///
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Navigation {
    /// Widget is not reachable with normal keyboard or mouse navigation.
    None,
    /// Focus is locked to stay with this widget. No mouse or keyboard navigation
    /// can change that.
    Lock,
    /// Widget is not reachable with keyboard navigation, but can be focused with the mouse.
    Mouse,
    /// Widget cannot be reached with normal keyboard navigation, but can be left.
    /// (e.g. Tabs, Split-Divider)
    Leave,
    /// Widget can be reached with normal keyboard navigation, but not left.
    /// (e.g. TextArea)
    Reach,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// backward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveFront,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// forward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveBack,
    /// Widget can be reached and left with normal keyboard navigation.
    #[default]
    Regular,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocus {
    /// Build the focus-structure for the container.
    fn build(&self, builder: &mut FocusBuilder) {
        builder.add_widget(self.focus(), self.area(), self.area_z(), self.navigable())
    }

    /// Access to the flag for the rest.
    fn focus(&self) -> FocusFlag;

    /// Area for mouse focus.
    ///
    /// This area shouldn't overlap with areas returned by other widgets.
    /// If it does, the widget should use `z_areas()` for clarification.
    /// Otherwise, the areas are searched in order of addition.
    fn area(&self) -> Rect;

    /// Z value for the area.
    ///
    /// When testing for mouse interactions the z-value is taken into
    /// consideration too.
    fn area_z(&self) -> u16 {
        0
    }

    /// Declares how the widget interacts with focus.
    ///
    /// Default is [Navigation::Regular].
    fn navigable(&self) -> Navigation {
        Navigation::Regular
    }

    /// Focused?
    fn is_focused(&self) -> bool {
        self.focus().get()
    }

    /// Just lost focus.
    fn lost_focus(&self) -> bool {
        self.focus().lost()
    }

    /// Just gained focus.
    fn gained_focus(&self) -> bool {
        self.focus().gained()
    }
}

/// Is this a container widget.
pub trait FocusContainer {
    /// Build the focus-structure for the container.
    fn build(&self, builder: &mut FocusBuilder);

    /// Returns the container-flag.
    ///
    /// Will be used to collect the state of all widgets in the container.
    fn container(&self) -> Option<ContainerFlag> {
        None
    }

    /// Area of the container for mouse-events.
    fn area(&self) -> Rect {
        Rect::default()
    }

    /// Z value for the area.
    ///
    /// When testing for mouse interactions the z-value is taken into
    /// consideration too.
    fn area_z(&self) -> u16 {
        0
    }

    /// Focused?
    fn is_container_focused(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.get()
        } else {
            false
        }
    }

    /// Just lost focus.
    fn container_lost_focus(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.lost()
        } else {
            false
        }
    }

    /// Just gained focus.
    fn container_gained_focus(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.gained()
        } else {
            false
        }
    }
}

impl FocusContainer for () {
    fn build(&self, _: &mut FocusBuilder) {}
}

impl Debug for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusFlag")
            .field("name", &self.name())
            .field("focus", &self.get())
            .field("gained", &self.gained())
            .field("lost", &self.lost())
            .finish()
    }
}

impl Debug for ContainerFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerFlag")
            .field("name", &self.name())
            .field("focus", &self.get())
            .field("gained", &self.gained())
            .field("lost", &self.lost())
            .finish()
    }
}

impl FocusFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return an identity value.
    fn focus_id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    /// Create a named flag.
    pub fn named(name: &str) -> Self {
        Self(Rc::new(FocusFlagCore::named(name)))
    }

    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.0.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, focus: bool) {
        self.0.focus.set(focus);
    }

    /// Get the field-name.
    #[inline]
    pub fn name(&self) -> &str {
        self.0.name.as_ref()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.0.lost.get()
    }

    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.0.focus.set(false);
        self.0.lost.set(false);
        self.0.gained.set(false);
    }
}

impl ContainerFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a named flag.
    pub fn named(name: &str) -> Self {
        Self(Rc::new(FocusFlagCore::named(name)))
    }

    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.0.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, focus: bool) {
        self.0.focus.set(focus);
    }

    /// Get the field-name.
    #[inline]
    pub fn name(&self) -> &str {
        self.0.name.as_ref()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.0.lost.get()
    }

    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.0.focus.set(false);
        self.0.lost.set(false);
        self.0.gained.set(false);
    }
}

impl FocusFlagCore {
    pub(crate) fn named(name: &str) -> Self {
        Self {
            name: name.into(),
            focus: Cell::new(false),
            gained: Cell::new(false),
            lost: Cell::new(false),
        }
    }
}

/// Adapter for widgets that don't use this library.
///
/// This can be constructed on the fly, if you have the necessary parts.
/// You will at least need a FocusFlag stored parallel to the widget state.
#[derive(Debug, Default)]
pub struct FocusAdapter {
    pub focus: FocusFlag,
    pub area: Rect,
    pub area_z: u16,
    pub navigation: Navigation,
}

impl HasFocus for FocusAdapter {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn area_z(&self) -> u16 {
        self.area_z
    }

    fn navigable(&self) -> Navigation {
        self.navigation
    }
}

/// Adapter for widgets that don't use this library.
///
/// This can be constructed on the fly, if you have the necessary parts.
/// You will at least need a ContainerFlag stored parallel to the widget state.
pub struct ContainerAdapter<'a> {
    pub container: ContainerFlag,
    pub build_fn: &'a dyn Fn(&mut FocusBuilder),
    pub area: Rect,
    pub area_z: u16,
}

impl Debug for ContainerAdapter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerAdapter")
            .field("container", &self.container)
            .field("build_fn", &"..dyn..")
            .field("area", &self.area)
            .field("area_z", &self.area_z)
            .finish()
    }
}

impl Default for ContainerAdapter<'_> {
    fn default() -> Self {
        Self {
            build_fn: &|_builder| {},
            container: Default::default(),
            area: Default::default(),
            area_z: 0,
        }
    }
}

impl FocusContainer for ContainerAdapter<'_> {
    fn build(&self, builder: &mut FocusBuilder) {
        (self.build_fn)(builder);
    }

    fn container(&self) -> Option<ContainerFlag> {
        Some(self.container.clone())
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn area_z(&self) -> u16 {
        self.area_z
    }
}

/// Does a match on the state struct of a widget. If `widget_state.lost_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_lost;
///
/// on_lost!(
///     state.field1 => {
///         // do checks
///     },
///     state.field2 => {
///         // do checks
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_lost {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocus;
        $(if $field.lost_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If `widget_state.gained_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_gained;
///
/// on_gained!(
///     state.field1 => {
///         // do prep
///     },
///     state.field2 => {
///         // do prep
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_gained {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocus;
        $(if $field.gained_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If
/// `widget_state.is_focused()` is true the block is executed.
/// There is a `_` branch too, that is evaluated if none of the
/// given widget-states has the focus.
///
/// This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::match_focus;
///
/// let res = match_focus!(
///     state.field1 => {
///         // do this
///         true
///     },
///     state.field2 => {
///         // do that
///         true
///     },
///     _ => {
///         false
///     }
/// );
///
/// if res {
///     // react
/// }
/// ```
///
#[macro_export]
macro_rules! match_focus {
    ($($field:expr => $block:expr),* $(, _ => $final:expr)?) => {{
        use $crate::HasFocus;
        if false {
            unreachable!();
        }
        $(else if $field.is_focused() { $block })*
        $(else { $final })?
    }};
}
