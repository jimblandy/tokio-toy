trait Future {
    type Item;
    type Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error>;
    ...
}

type Poll<T, E> = Result<Async<T>, E>;

enum Async<T> {
    Ready(T),
    NotReady
}

struct FutureResult<T, E>(Option<Result<T, E>>);

impl<T, E> Future for FutureResult<T, E> {
    type Item = T;
    type Error = E;
    fn poll(&mut self) -> Poll<T, E> {
        match self.0.take() {
            None => panic!("polled FutureResult twice"),
            Some(Ok(v)) => Ok(Async::Ready(v)),
            Some(Err(e)) => Err(e)
        }
    }
}

fn result<T, E>result(r: Result<T, E>) -> FutureResult<T, E> {
    FutureResult(Some(r))
}

fn ok<T, E>(value: T) -> FutureResult<T, E> {
    result(Ok(value))
}

fn err<T, E>(e: E) -> FutureResult<T, E> {
    result(Err(e))
}


struct Map<A, F> where A: Future {
    future: A,
    closure: Option<F>,
}

fn map<A, F>(self, closure: F) -> Map<A, F> {
    Map { future: self, Some(closure) }
}


impl<F, T, E, A> Future<U, E> for Map<A, F>
    where A: Future<T, E>,
          F: FnOnce(T) -> U
{
    type Item = U;
    type Error = E;
    fn poll(&mut self) -> Poll<U, E> {
        match self.future.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(v)) =>
                Ok(Async::Ready(self.closure.take().expect("polled map twice")(v))),
            Err(e) => Err(e)
        }
    }
}


