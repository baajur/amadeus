use super::{Consumer, ConsumerMulti, DistributedIterator, DistributedIteratorMulti};
use sum::Sum2;

impl<A: DistributedIterator, B: DistributedIterator<Item = A::Item>> DistributedIterator
	for Sum2<A, B>
{
	type Item = A::Item;
	type Task = Sum2<A::Task, B::Task>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Sum2::A(i) => i.size_hint(),
			Sum2::B(i) => i.size_hint(),
		}
	}
	fn next_task(&mut self) -> Option<Self::Task> {
		match self {
			Sum2::A(i) => i.next_task().map(Sum2::A),
			Sum2::B(i) => i.next_task().map(Sum2::B),
		}
	}
}

impl<
		A: DistributedIteratorMulti<Source>,
		B: DistributedIteratorMulti<Source, Item = A::Item>,
		Source,
	> DistributedIteratorMulti<Source> for Sum2<A, B>
{
	type Item = A::Item;
	type Task = Sum2<A::Task, B::Task>;

	fn task(&self) -> Self::Task {
		match self {
			Sum2::A(i) => Sum2::A(i.task()),
			Sum2::B(i) => Sum2::B(i.task()),
		}
	}
}

impl<A: Consumer, B: Consumer<Item = A::Item>> Consumer for Sum2<A, B> {
	type Item = A::Item;

	fn run(self, i: &mut impl FnMut(Self::Item) -> bool) -> bool {
		match self {
			Sum2::A(task) => task.run(i),
			Sum2::B(task) => task.run(i),
		}
	}
}

impl<A: ConsumerMulti<Source>, B: ConsumerMulti<Source, Item = A::Item>, Source>
	ConsumerMulti<Source> for Sum2<A, B>
{
	type Item = A::Item;

	fn run(&self, source: Source, i: &mut impl FnMut(Self::Item) -> bool) -> bool {
		match self {
			Sum2::A(task) => task.run(source, i),
			Sum2::B(task) => task.run(source, i),
		}
	}
}
