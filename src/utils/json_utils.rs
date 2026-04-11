/// these utils are part of an old api of JsonClient I don't know If I need them anymore
// // useful but obselete primitive
// pub struct ReturnAsJsonMap<T>(pub Vec<(String, T)>);
//
// // a common pattern is you have array of fragments and you
// // want to build them as a map
// impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for ReturnAsJsonMap<T>
// where
//     T: SelectOneJsonFragment<S>,
// {
//     fn on_select(&mut self, st: &mut SelectSt<S>) {
//         self.0.iter_mut().for_each(|e| e.1.on_select(st))
//     }
//
//     fn from_row(&mut self, row: &<S>::Row) {
//         self.0.iter_mut().for_each(|e| e.1.from_row(row))
//     }
//
//     fn sub_op<'this>(
//         &'this mut self,
//         pool: Pool<S>,
//     ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
//         Box::pin(async move {
//             for item in self.0.iter_mut() {
//                 item.1.sub_op(pool.clone()).await
//             }
//         })
//     }
//
//     fn take(self: Box<Self>) -> serde_json::Value {
//         let mut map = serde_json::Map::new();
//         self.0.into_iter().for_each(|e| {
//             map.insert(e.0, Box::new(e.1).take());
//         });
//         map.into()
//     }
// }

pub fn from_map(map: &mut Map<String, Value>, from: &Vec<&'static str>) -> Option<Value> {
    if from.len() == 1 {
        return Some(map.remove_entry(from[0])?.1);
    } else if from.len() == 2 {
        return Some(
            map.get_mut(from[0])?
                .as_object_mut()?
                .remove_entry(from[1])?
                .1,
        );
    } else if from.len() == 3 {
        return Some(
            map.get_mut(from[0])?
                .as_object_mut()?
                .get_mut(from[1])?
                .as_object_mut()?
                .remove_entry(from[1])?
                .1,
        );
    } else {
        panic!(
            "accessor of more that 3 \"{from:?}\" can be supported via recursive function but need unit testing to make sure it is valid"
        );
    }
}

pub fn map_is_empty(map: &mut Map<String, Value>) -> bool {
    if map.len() == 0 {
        true
    } else {
        map.values_mut().any(|e| {
            if let Some(e) = e.as_object_mut() {
                map_is_empty(e)
            } else {
                false
            }
        })
    }
}
