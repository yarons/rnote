pub mod background;
pub mod format;
pub mod selection;

use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    pens::eraser::Eraser,
    sheet::selection::Selection,
    strokes::{self, compose, StrokeBehaviour, StrokeStyle},
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage},
    utils::{self, FileType},
};

use self::{background::Background, format::Format};

use gtk4::{gdk, gio, glib, graphene, gsk, prelude::*, subclass::prelude::*, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

mod imp {
    use std::cell::Cell;
    use std::{cell::RefCell, rc::Rc};

    use gtk4::{glib, subclass::prelude::*};

    use crate::sheet::selection::Selection;
    use crate::strokes;

    use super::{Background, Format};

    #[derive(Debug)]
    pub struct Sheet {
        pub strokes: Rc<RefCell<Vec<strokes::StrokeStyle>>>,
        pub strokes_trash: Rc<RefCell<Vec<strokes::StrokeStyle>>>,
        pub selection: Selection,
        pub format: Rc<RefCell<Format>>,
        pub background: Rc<RefCell<Background>>,
        pub x: Cell<i32>,
        pub y: Cell<i32>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
        pub autoexpand_height: Cell<bool>,
        pub format_borders: Cell<bool>,
        pub padding_bottom: Cell<i32>,
    }

    impl Default for Sheet {
        fn default() -> Self {
            Self {
                strokes: Rc::new(RefCell::new(Vec::new())),
                strokes_trash: Rc::new(RefCell::new(Vec::new())),
                selection: Selection::new(),
                format: Rc::new(RefCell::new(Format::default())),
                background: Rc::new(RefCell::new(Background::default())),
                x: Cell::new(0),
                y: Cell::new(0),
                width: Cell::new(Format::default().width),
                height: Cell::new(Format::default().height),
                autoexpand_height: Cell::new(true),
                format_borders: Cell::new(true),
                padding_bottom: Cell::new(Format::default().height),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sheet {
        const NAME: &'static str = "Sheet";
        type Type = super::Sheet;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Sheet {}
}

glib::wrapper! {
    pub struct Sheet(ObjectSubclass<imp::Sheet>);
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Sheet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Sheet", 2)?;
        state.serialize_field("strokes", &*self.strokes().borrow())?;
        state.serialize_field("strokes_trash", &*self.strokes_trash().borrow())?;
        state.serialize_field("selection", &self.selection())?;
        state.serialize_field("format", &self.format())?;
        state.serialize_field("background", &self.background())?;
        state.serialize_field("x", &self.x())?;
        state.serialize_field("y", &self.y())?;
        state.serialize_field("width", &self.width())?;
        state.serialize_field("height", &self.height())?;
        state.serialize_field("autoexpand_height", &self.autoexpand_height())?;
        state.serialize_field("format_borders", &self.format_borders())?;
        state.serialize_field("padding_bottom", &self.padding_bottom())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Sheet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            strokes,
            strokes_trash,
            selection,
            format,
            background,
            x,
            y,
            width,
            height,
            autoexpand_height,
            format_borders,
            padding_bottom,
        }

        struct SheetVisitor;
        impl<'de> Visitor<'de> for SheetVisitor {
            type Value = Sheet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Sheet")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let strokes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let strokes_trash = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let selection: Selection = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let format = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let background = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;
                let autoexpand_height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(9, &self))?;
                let format_borders = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(10, &self))?;
                let padding_bottom = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(11, &self))?;

                let sheet = Sheet::new();
                *sheet.strokes().borrow_mut() = strokes;
                *sheet.strokes_trash().borrow_mut() = strokes_trash;
                *sheet.selection().strokes().borrow_mut() = selection.strokes().borrow().clone();
                *sheet.selection().bounds().borrow_mut() = *selection.bounds().borrow();
                sheet.selection().set_shown(selection.shown());
                *sheet.format().borrow_mut() = format;
                *sheet.background().borrow_mut() = background;
                sheet.set_x(x);
                sheet.set_y(y);
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_autoexpand_height(autoexpand_height);
                sheet.set_format_borders(format_borders);
                sheet.set_padding_bottom(padding_bottom);

                // Register the custom templates into the brushes (serde does not yet support post-deserialize hooks, see https://github.com/paritytech/parity-scale-codec/issues/280)
                match StrokeStyle::register_custom_templates(&mut *sheet.strokes().borrow_mut()) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }
                match StrokeStyle::register_custom_templates(
                    &mut *sheet.strokes_trash().borrow_mut(),
                ) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }
                match StrokeStyle::register_custom_templates(
                    &mut *sheet.selection().strokes().borrow_mut(),
                ) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }

                Ok(sheet)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut strokes = None;
                let mut strokes_trash = None;
                let mut selection = None;
                let mut format = None;
                let mut background = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                let mut autoexpand_height = None;
                let mut format_borders = None;
                let mut padding_bottom = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::strokes => {
                            if strokes.is_some() {
                                return Err(de::Error::duplicate_field("strokes"));
                            }
                            strokes = Some(map.next_value()?);
                        }
                        Field::strokes_trash => {
                            if strokes_trash.is_some() {
                                return Err(de::Error::duplicate_field("strokes_trash"));
                            }
                            strokes_trash = Some(map.next_value()?);
                        }
                        Field::selection => {
                            if selection.is_some() {
                                return Err(de::Error::duplicate_field("selection"));
                            }
                            selection = Some(map.next_value()?);
                        }
                        Field::format => {
                            if format.is_some() {
                                return Err(de::Error::duplicate_field("format"));
                            }
                            format = Some(map.next_value()?);
                        }
                        Field::background => {
                            if background.is_some() {
                                return Err(de::Error::duplicate_field("background"));
                            }
                            background = Some(map.next_value()?);
                        }
                        Field::x => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                        Field::width => {
                            if width.is_some() {
                                return Err(de::Error::duplicate_field("width"));
                            }
                            width = Some(map.next_value()?);
                        }
                        Field::height => {
                            if height.is_some() {
                                return Err(de::Error::duplicate_field("height"));
                            }
                            height = Some(map.next_value()?);
                        }
                        Field::autoexpand_height => {
                            if autoexpand_height.is_some() {
                                return Err(de::Error::duplicate_field("autoexpand_height"));
                            }
                            autoexpand_height = Some(map.next_value()?);
                        }
                        Field::format_borders => {
                            if format_borders.is_some() {
                                return Err(de::Error::duplicate_field("format_borders"));
                            }
                            format_borders = Some(map.next_value()?);
                        }
                        Field::padding_bottom => {
                            if padding_bottom.is_some() {
                                return Err(de::Error::duplicate_field("padding_bottom"));
                            }
                            padding_bottom = Some(map.next_value()?);
                        }
                    }
                }

                let strokes = strokes.ok_or_else(|| de::Error::missing_field("strokes"))?;
                let strokes_trash =
                    strokes_trash.ok_or_else(|| de::Error::missing_field("strokes_trash"))?;
                let selection: Selection =
                    selection.ok_or_else(|| de::Error::missing_field("selection"))?;
                let format: Format = format.ok_or_else(|| de::Error::missing_field("format"))?;
                let background: Background =
                    background.ok_or_else(|| de::Error::missing_field("background"))?;
                let x: i32 = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y: i32 = y.ok_or_else(|| de::Error::missing_field("y"))?;
                let width: i32 = width.ok_or_else(|| de::Error::missing_field("width"))?;
                let height: i32 = height.ok_or_else(|| de::Error::missing_field("height"))?;
                let autoexpand_height: bool = autoexpand_height
                    .ok_or_else(|| de::Error::missing_field("autoexpand_height"))?;
                let format_borders: bool =
                    format_borders.ok_or_else(|| de::Error::missing_field("format_borders"))?;
                let padding_bottom: i32 =
                    padding_bottom.ok_or_else(|| de::Error::missing_field("padding_bottom"))?;

                let sheet = Sheet::new();
                *sheet.strokes().borrow_mut() = strokes;
                *sheet.strokes_trash().borrow_mut() = strokes_trash;
                *sheet.selection().strokes().borrow_mut() = selection.strokes().borrow().clone();
                *sheet.selection().bounds().borrow_mut() = *selection.bounds().borrow();
                sheet.selection().set_shown(selection.shown());
                *sheet.format().borrow_mut() = format;
                *sheet.background().borrow_mut() = background;
                sheet.set_x(x);
                sheet.set_y(y);
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_autoexpand_height(autoexpand_height);
                sheet.set_format_borders(format_borders);
                sheet.set_padding_bottom(padding_bottom);

                // Register the custom templates into the brushes (serde does not yet support post-deserialize hooks, see https://github.com/paritytech/parity-scale-codec/issues/280)
                match StrokeStyle::register_custom_templates(&mut *sheet.strokes().borrow_mut()) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }
                match StrokeStyle::register_custom_templates(
                    &mut *sheet.strokes_trash().borrow_mut(),
                ) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }
                match StrokeStyle::register_custom_templates(
                    &mut *sheet.selection().strokes().borrow_mut(),
                ) {
                    Err(e) => {
                        return Err(de::Error::custom(format!(
                            "failed to register custom template for sheet strokes, {:?}",
                            e
                        )));
                    }
                    Ok(()) => {}
                }

                Ok(sheet)
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "strokes",
            "strokes_trash",
            "selection",
            "format",
            "background",
            "x",
            "y",
            "width",
            "height",
            "autoexpand_height",
            "format_borders",
            "padding_bottom",
        ];
        deserializer.deserialize_struct("Sheet", FIELDS, SheetVisitor)
    }
}

impl Sheet {
    pub const SHADOW_WIDTH: f64 = 15.0;
    pub fn new() -> Self {
        let sheet: Sheet = glib::Object::new(&[]).expect("Failed to create Sheet");
        sheet
    }

    pub fn strokes(&self) -> Rc<RefCell<Vec<StrokeStyle>>> {
        imp::Sheet::from_instance(self).strokes.clone()
    }

    pub fn strokes_trash(&self) -> Rc<RefCell<Vec<StrokeStyle>>> {
        imp::Sheet::from_instance(self).strokes_trash.clone()
    }

    pub fn selection(&self) -> Selection {
        imp::Sheet::from_instance(self).selection.clone()
    }

    pub fn x(&self) -> i32 {
        imp::Sheet::from_instance(self).x.get()
    }

    pub fn set_x(&self, x: i32) {
        imp::Sheet::from_instance(self).x.set(x)
    }

    pub fn y(&self) -> i32 {
        imp::Sheet::from_instance(self).y.get()
    }

    pub fn set_y(&self, y: i32) {
        imp::Sheet::from_instance(self).y.set(y)
    }

    pub fn width(&self) -> i32 {
        imp::Sheet::from_instance(self).width.get()
    }

    pub fn set_width(&self, width: i32) {
        imp::Sheet::from_instance(self).width.set(width);
    }

    pub fn height(&self) -> i32 {
        imp::Sheet::from_instance(self).height.get()
    }

    pub fn set_height(&self, height: i32) {
        imp::Sheet::from_instance(self).height.set(height);
    }

    pub fn autoexpand_height(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.autoexpand_height.get()
    }

    pub fn set_autoexpand_height(&self, autoexpand_height: bool) {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.autoexpand_height.set(autoexpand_height);

        if autoexpand_height {
            priv_.padding_bottom.set(2 * priv_.format.borrow().height);
            self.resize();
        } else {
            priv_.padding_bottom.set(0);
            self.fit_to_format();
        }
    }

    pub fn format(&self) -> Rc<RefCell<Format>> {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format.clone()
    }

    pub fn format_borders(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format_borders.get()
    }

    pub fn set_format_borders(&self, format_borders: bool) {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format_borders.set(format_borders);
    }

    pub fn padding_bottom(&self) -> i32 {
        imp::Sheet::from_instance(self).padding_bottom.get()
    }

    pub fn set_padding_bottom(&self, padding_bottom: i32) {
        imp::Sheet::from_instance(self)
            .padding_bottom
            .set(padding_bottom);
    }

    pub fn background(&self) -> Rc<RefCell<Background>> {
        imp::Sheet::from_instance(self).background.clone()
    }

    // returns true if resizing is needed
    pub fn undo_last_stroke(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(removed_stroke) = priv_.strokes.borrow_mut().pop() {
            priv_.strokes_trash.borrow_mut().push(removed_stroke);
        }
        self.resize()
    }

    // returns true if resizing is needed
    pub fn redo_last_stroke(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(restored_stroke) = priv_.strokes_trash.borrow_mut().pop() {
            priv_.strokes.borrow_mut().push(restored_stroke);
        }
        self.resize()
    }

    // returns true if resizing is needed
    pub fn remove_colliding_strokes(&self, eraser: &Eraser) -> bool {
        let priv_ = imp::Sheet::from_instance(self);

        let eraser_bounds = p2d::bounding_volume::AABB::new(
            na::Point2::from(
                eraser.current_input.pos() - na::vector![eraser.width / 2.0, eraser.width / 2.0],
            ),
            na::Point2::from(
                eraser.current_input.pos() + na::vector![eraser.width / 2.0, eraser.width / 2.0],
            ),
        );

        let mut removed_strokes: Vec<strokes::StrokeStyle> = Vec::new();

        priv_.strokes.borrow_mut().retain(|stroke| {
            match stroke {
                strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                    // First check markerstroke bounds, and if true zoom in and check hitbox
                    if eraser_bounds.intersects(&markerstroke.bounds) {
                        for hitbox_elem in markerstroke.hitbox.iter() {
                            if eraser_bounds.intersects(hitbox_elem) {
                                removed_strokes.push(stroke.clone());
                                return false;
                            }
                        }
                    }
                }
                strokes::StrokeStyle::BrushStroke(brushstroke) => {
                    // First check markerstroke bounds, and if true zoom in and check hitbox
                    if eraser_bounds.intersects(&brushstroke.bounds) {
                        for hitbox_elem in brushstroke.hitbox.iter() {
                            if eraser_bounds.intersects(hitbox_elem) {
                                removed_strokes.push(stroke.clone());
                                return false;
                            }
                        }
                    }
                }
                strokes::StrokeStyle::VectorImage(vectorimage) => {
                    if eraser_bounds.intersects(&vectorimage.bounds) {
                        removed_strokes.push(stroke.clone());
                        return false;
                    }
                }
                strokes::StrokeStyle::BitmapImage(bitmapimage) => {
                    if eraser_bounds.intersects(&bitmapimage.bounds) {
                        removed_strokes.push(stroke.clone());
                        return false;
                    }
                }
            }

            true
        });
        priv_
            .strokes_trash
            .borrow_mut()
            .append(&mut removed_strokes);

        self.resize()
    }

    // Returns true if resizing is needed
    pub fn clear(&self) {
        let priv_ = imp::Sheet::from_instance(self);

        priv_.strokes.borrow_mut().clear();
        priv_.strokes_trash.borrow_mut().clear();
        priv_.selection.strokes().borrow_mut().clear();

        if self.autoexpand_height() {
            self.set_padding_bottom(2 * priv_.format.borrow().height);
            self.resize();
        } else {
            self.set_padding_bottom(0);
            self.fit_to_format();
        }
    }

    // In format (width, height, dpi)
    pub fn change_format(&self, format_tuple: (i32, i32, i32)) {
        let priv_ = imp::Sheet::from_instance(self);

        *priv_.format.borrow_mut() = Format {
            width: format_tuple.0,
            height: format_tuple.1,
            dpi: format_tuple.2,
        };

        self.set_width(priv_.format.borrow().width);
        self.set_height(priv_.format.borrow().height);
        self.set_padding_bottom(2 * priv_.format.borrow().height);
        // Ignoring return parameter of resize() because resizing is needed either way.
        self.resize();
    }

    pub fn fit_to_format(&self) {
        let priv_ = imp::Sheet::from_instance(self);

        let new_height = self.calc_height();
        self.set_height(
            (new_height as f64 / priv_.format.borrow().height as f64).ceil() as i32
                * priv_.format.borrow().height,
        );
    }

    // Resizing the sheet format height and returning true if window resizing is needed
    pub fn resize(&self) -> bool {
        if !self.autoexpand_height() {
            return false;
        }

        let mut resize = false;
        let new_height = self.calc_height();

        if new_height != self.height() {
            resize = true;
            self.set_height(new_height);
        }

        resize
    }

    pub fn calc_height(&self) -> i32 {
        let priv_ = imp::Sheet::from_instance(self);

        let new_height = if let Some(stroke) =
            priv_.strokes.borrow().iter().max_by_key(|&stroke| {
                stroke.bounds().maxs[1].round() as i32 + self.padding_bottom()
            }) {
            // max_by_key() returns the element, so we need to extract the height again
            stroke.bounds().maxs[1].round() as i32 + self.padding_bottom()
        } else {
            // Iterator empty so resizing to format height
            priv_.format.borrow().height
        };

        new_height
    }

    pub fn remove_strokes(&self, indices: Vec<usize>) {
        let priv_ = imp::Sheet::from_instance(self);

        for i in indices.iter() {
            let mut index: Option<usize> = None;
            if priv_.strokes.borrow().get(*i).is_some() {
                index = Some(*i);
            } else {
                log::error!(
                    "remove_strokes() failed at index {}, index is out of bounds",
                    i
                );
            }
            if let Some(index) = index {
                priv_.strokes.borrow_mut().remove(index);
            }
        }
    }

    pub fn draw(&self, scalefactor: f64, snapshot: &Snapshot) {
        let priv_ = imp::Sheet::from_instance(self);

        let sheet_bounds = graphene::Rect::new(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );
        let sheet_bounds_scaled = graphene::Rect::new(
            self.x() as f32 * scalefactor as f32,
            self.y() as f32 * scalefactor as f32,
            self.width() as f32 * scalefactor as f32,
            self.height() as f32 * scalefactor as f32,
        );

        self.draw_shadow(&sheet_bounds, Self::SHADOW_WIDTH, scalefactor, snapshot);

        snapshot.push_clip(&sheet_bounds_scaled);

        priv_
            .background
            .borrow()
            .draw(&snapshot, &sheet_bounds_scaled);

        if self.format_borders() {
            priv_
                .format
                .borrow()
                .draw(self.height(), &snapshot, scalefactor);
        }

        StrokeStyle::draw_strokes(&priv_.strokes.borrow(), &snapshot);

        snapshot.pop();
    }

    pub fn draw_shadow(
        &self,
        bounds: &graphene::Rect,
        width: f64,
        scalefactor: f64,
        snapshot: &Snapshot,
    ) {
        let width = width * scalefactor;
        let shadow_color = gdk::RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.5,
        };
        let corner_radius = graphene::Size::new(width as f32, width as f32);

        let rounded_rect = gsk::RoundedRect::new(
            bounds.clone().scale(scalefactor as f32, scalefactor as f32),
            corner_radius.clone(),
            corner_radius.clone(),
            corner_radius.clone(),
            corner_radius,
        );

        snapshot.append_outset_shadow(
            &rounded_rect,
            &shadow_color,
            0.0,
            0.0,
            width as f32,
            width as f32,
        );
    }

    pub fn open_sheet(&self, file: &gio::File) -> Result<(), Box<dyn Error>> {
        let sheet: Sheet = serde_json::from_str(&utils::load_file_contents(&file)?)?;

        *self.strokes().borrow_mut() = sheet.strokes().borrow().clone();
        *self.strokes_trash().borrow_mut() = sheet.strokes().borrow_mut().clone();
        *self.selection().strokes().borrow_mut() = sheet.selection().strokes().borrow().clone();
        *self.selection().bounds().borrow_mut() = *sheet.selection().bounds().borrow();
        self.selection().set_shown(sheet.selection().shown());
        *self.format().borrow_mut() = sheet.format().borrow().clone();
        *self.background().borrow_mut() = sheet.background().borrow().clone();
        self.set_x(sheet.x());
        self.set_y(sheet.y());
        self.set_width(sheet.width());
        self.set_height(sheet.height());
        self.set_autoexpand_height(sheet.autoexpand_height());
        self.set_format_borders(sheet.format_borders());
        self.set_padding_bottom(sheet.padding_bottom());

        StrokeStyle::complete_all_strokes(&mut *self.strokes().borrow_mut());
        StrokeStyle::complete_all_strokes(&mut *self.strokes_trash().borrow_mut());
        StrokeStyle::complete_all_strokes(&mut *self.selection().strokes().borrow_mut());
        Ok(())
    }

    pub fn save_sheet(&self, file: &gio::File) -> Result<(), Box<dyn Error>> {
        match FileType::lookup_file_type(file) {
            FileType::Rnote => {
                let json_output = serde_json::to_string(self)?;
                let output_stream = file.replace::<gio::Cancellable>(
                    None,
                    false,
                    gio::FileCreateFlags::REPLACE_DESTINATION,
                    None,
                )?;

                output_stream.write::<gio::Cancellable>(json_output.as_bytes(), None)?;
                output_stream.close::<gio::Cancellable>(None)?;
            }
            _ => {
                log::error!("invalid output file type for saving sheet in native format");
            }
        }
        Ok(())
    }

    pub fn export_sheet_as_svg(&self, file: gio::File) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        let mut data = String::new();
        for stroke in &*priv_.strokes.borrow() {
            let data_entry = stroke.gen_svg_data(na::vector![0.0, 0.0])?;

            data.push_str(&data_entry);
        }

        let sheet_bounds = p2d::bounding_volume::AABB::new(
            na::point![f64::from(self.x()), f64::from(self.y())],
            na::point![
                f64::from(self.x() + self.width()),
                f64::from(self.y() + self.height())
            ],
        );

        data = compose::wrap_svg(
            data.as_str(),
            Some(sheet_bounds),
            Some(sheet_bounds),
            true,
            true,
        );

        let output_stream = file.replace::<gio::Cancellable>(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            None,
        )?;
        output_stream.write::<gio::Cancellable>(data.as_bytes(), None)?;
        output_stream.close::<gio::Cancellable>(None)?;

        Ok(())
    }

    pub fn import_file_as_svg(
        &self,
        pos: na::Vector2<f64>,
        file: &gio::File,
    ) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        let svg = utils::load_file_contents(&file)?;

        priv_
            .strokes
            .borrow_mut()
            .append(&mut priv_.selection.remove_strokes());

        let vector_image = VectorImage::import_from_svg(svg.as_str(), pos).unwrap();
        priv_
            .selection
            .push_to_selection(strokes::StrokeStyle::VectorImage(vector_image));

        Ok(())
    }

    pub fn import_file_as_bitmapimage(
        &self,
        pos: na::Vector2<f64>,
        file: &gio::File,
    ) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        priv_
            .strokes
            .borrow_mut()
            .append(&mut priv_.selection.remove_strokes());

        let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
        let bitmapimage = BitmapImage::import_from_image_bytes(&file_bytes, pos).unwrap();

        priv_
            .selection
            .push_to_selection(strokes::StrokeStyle::BitmapImage(bitmapimage));

        Ok(())
    }
}