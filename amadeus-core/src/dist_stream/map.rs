use futures::{pin_mut, Stream};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{
	pin::Pin, task::{Context, Poll}
};

use super::{
	DistributedPipe, DistributedStream, PipeTask, PipeTaskAsync, StreamTask, StreamTaskAsync
};
use crate::{
	pool::ProcessSend, sink::{Sink, SinkMap}
};

#[must_use]
pub struct Map<I, F> {
	i: I,
	f: F,
}
impl<I, F> Map<I, F> {
	pub(crate) fn new(i: I, f: F) -> Self {
		Self { i, f }
	}
}

impl<I: DistributedStream, F, R> DistributedStream for Map<I, F>
where
	F: FnMut(I::Item) -> R + Clone + ProcessSend,
{
	type Item = R;
	type Task = MapTask<I::Task, F>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.i.size_hint()
	}
	fn next_task(&mut self) -> Option<Self::Task> {
		self.i.next_task().map(|task| {
			let f = self.f.clone();
			MapTask { task, f }
		})
	}
}

impl<I: DistributedPipe<Source>, F, R, Source> DistributedPipe<Source> for Map<I, F>
where
	F: FnMut(<I as DistributedPipe<Source>>::Item) -> R + Clone + ProcessSend,
{
	type Item = R;
	type Task = MapTask<I::Task, F>;

	fn task(&self) -> Self::Task {
		let task = self.i.task();
		let f = self.f.clone();
		MapTask { task, f }
	}
}

#[pin_project]
#[derive(Serialize, Deserialize)]
pub struct MapTask<C, F> {
	#[pin]
	task: C,
	f: F,
}

impl<C: StreamTask, F, R> StreamTask for MapTask<C, F>
where
	F: FnMut(C::Item) -> R + Clone,
{
	type Item = R;
	type Async = MapTask<C::Async, F>;
	fn into_async(self) -> Self::Async {
		MapTask {
			task: self.task.into_async(),
			f: self.f,
		}
	}
}
impl<C: PipeTask<Source>, F, R, Source> PipeTask<Source> for MapTask<C, F>
where
	F: FnMut(C::Item) -> R + Clone,
{
	type Item = R;
	type Async = MapTask<C::Async, F>;
	fn into_async(self) -> Self::Async {
		MapTask {
			task: self.task.into_async(),
			f: self.f,
		}
	}
}

impl<C: StreamTaskAsync, F, R> StreamTaskAsync for MapTask<C, F>
where
	F: FnMut(C::Item) -> R + Clone,
{
	type Item = R;

	fn poll_run(
		self: Pin<&mut Self>, cx: &mut Context, sink: Pin<&mut impl Sink<Self::Item>>,
	) -> Poll<()> {
		let mut self_ = self.project();
		let (task, f) = (self_.task, &mut self_.f);
		let sink = SinkMap::new(sink, |item| f(item));
		pin_mut!(sink);
		task.poll_run(cx, sink)
	}
}

impl<C: PipeTaskAsync<Source>, F, R, Source> PipeTaskAsync<Source> for MapTask<C, F>
where
	F: FnMut(<C as PipeTaskAsync<Source>>::Item) -> R + Clone,
{
	type Item = R;

	fn poll_run(
		self: Pin<&mut Self>, cx: &mut Context, stream: Pin<&mut impl Stream<Item = Source>>,
		sink: Pin<&mut impl Sink<Self::Item>>,
	) -> Poll<()> {
		let mut self_ = self.project();
		let (task, f) = (self_.task, &mut self_.f);
		let sink = SinkMap::new(sink, |item| f(item));
		pin_mut!(sink);
		task.poll_run(cx, stream, sink)
	}
}
