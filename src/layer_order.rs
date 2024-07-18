use tiled::{Layer, PropertyValue};

#[derive(Debug)]
pub struct LayerY {
    pub index: usize,
    pub y: i32,
}

/// Implements a convention of layer Y-order, e.g.
/// walls z100
/// walls z50
/// floor z0
/// floor z1
/// Greater z-order is drawn closer to in background, lower z-order is drawn in foreground.
/// All "floor" layers are drawn in background, before "walls".
#[derive(Debug)]
pub struct LayersOrder {
    indexes: Vec<LayerY>,
}

impl LayersOrder {
    /// Create it from an array of layer indexes & names
    pub fn new<'map>(layers: impl ExactSizeIterator<Item = Layer<'map>>) -> Self {
        fn layer_order(layer: Layer) -> i32 {
            match layer.properties.get("yorder") {
                Some(PropertyValue::IntValue(val)) => *val,
                _ => -1,
            }
        }

        let mut indexes: Vec<_> = layers
            .enumerate()
            .map(|(index, layer)| LayerY {
                index,
                y: layer_order(layer),
            })
            .collect();

        indexes.sort_by(|a, b| a.y.cmp(&b.y));

        Self { indexes }
    }

    /// Read the order of drawing layers
    pub fn order(&self) -> &Vec<LayerY> {
        &self.indexes
    }
}
