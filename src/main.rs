use doublelist::DoubleList;

// fn check() -> DoubleList<i32>{
//     let mut list : DoubleList<i32> = DoubleList::new();
//     list.insert_front(11);
//     list.insert_front(23);

//     list
// }

fn main() {
    let sample = [1,4,43];

    let mut list : DoubleList<i32> = DoubleList::new();
    for s in sample {
        list.insert_front(s);
    }

    list.print(3);

    let mut i = sample.len() - 1;
    for x in list.iter() {
        assert_eq!(*x, sample[i]);
        if i > 0 {
            i -= 1;
        }
    }
}
