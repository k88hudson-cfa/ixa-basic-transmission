use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::BTreeMap;

use frunk::hlist;
use frunk::hlist::{HCons, HNil};
use ixa::people::PersonProperty;
use ixa::{PersonId, PluginContext};

/// A trait representing a typed property/value pair
pub trait PropertyValuePair {
    type PropertyType: PersonProperty;
    fn value(&self) -> <Self::PropertyType as PersonProperty>::Value;
}

/// Generic wrapper for a concrete property/value
#[derive(Debug, Clone)]
pub struct PropertyValue<P: PersonProperty> {
    pub value: P::Value,
}

impl<P: PersonProperty> PropertyValuePair for PropertyValue<P> {
    type PropertyType = P;
    fn value(&self) -> P::Value {
        self.value.clone()
    }
}

/// Convert an HList of property/value pairs into a map of TypeId -> Value
pub trait ToOrderedMap {
    type Value: Ord;
    fn to_map(&self) -> BTreeMap<TypeId, Self::Value>;
}

impl ToOrderedMap for HNil {
    type Value = ();
    fn to_map(&self) -> BTreeMap<TypeId, Self::Value> {
        BTreeMap::new()
    }
}

impl<Head, Tail> ToOrderedMap for HCons<Head, Tail>
where
    Head: PropertyValuePair,
    <Head::PropertyType as PersonProperty>::Value: Ord + Clone + 'static,
    Tail: ToOrderedMap<Value = <Head::PropertyType as PersonProperty>::Value>,
{
    type Value = <Head::PropertyType as PersonProperty>::Value;

    fn to_map(&self) -> BTreeMap<TypeId, Self::Value> {
        let mut map = self.tail.to_map();
        map.insert(TypeId::of::<Head::PropertyType>(), self.head.value());
        map
    }
}

/// Core query type
#[derive(Debug)]
pub struct Query<Pairs> {
    pairs: Pairs,
}

impl<Pairs> Query<Pairs> {
    pub fn new(pairs: Pairs) -> Self {
        Self { pairs }
    }
}

/// Matching against person data
pub trait MatchPerson {
    fn match_person(&self, ctx: &impl PluginContext, person_id: PersonId) -> bool;
}

impl MatchPerson for HNil {
    fn match_person(&self, _ctx: &impl PluginContext, _person_id: PersonId) -> bool {
        true
    }
}

impl<Head, Tail> MatchPerson for HCons<Head, Tail>
where
    Head: PropertyValuePair,
    Tail: MatchPerson,
{
    fn match_person(&self, ctx: &impl PluginContext, person_id: PersonId) -> bool {
        let actual_value = ctx.get_person_property(
            person_id,
            <Head as PropertyValuePair>::PropertyType::get_instance(),
        );
        actual_value == self.head.value() && self.tail.match_person(ctx, person_id)
    }
}

/// Equality and ordering for queries
impl<Pairs> PartialEq for Query<Pairs>
where
    Pairs: ToOrderedMap,
    Pairs::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.pairs.to_map() == other.pairs.to_map()
    }
}

impl<Pairs> Eq for Query<Pairs>
where
    Pairs: ToOrderedMap,
    Pairs::Value: Eq,
{
}

impl<Pairs> PartialOrd for Query<Pairs>
where
    Pairs: ToOrderedMap,
    Pairs::Value: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Pairs> Ord for Query<Pairs>
where
    Pairs: ToOrderedMap,
    Pairs::Value: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.pairs.to_map().cmp(&other.pairs.to_map())
    }
}

/// Simple tuple-based constructors
impl From<()> for Query<HNil> {
    fn from(_: ()) -> Self {
        Query::new(HNil)
    }
}

impl<P: PersonProperty> From<(P, P::Value)> for Query<HCons<PropertyValue<P>, HNil>> {
    fn from(pair: (P, P::Value)) -> Self {
        Query::new(hlist![PropertyValue::<P> { value: pair.1 }])
    }
}

/// Convenience macro for inline query construction
#[macro_export]
macro_rules! query {
    ( $( ($ty:ty, $val:expr) ),+ $(,)? ) => {
        $crate::Query::new(frunk::hlist![
            $(
                $crate::PropertyValue::<$ty> { value: $val },
            )+
        ])
    };
}
