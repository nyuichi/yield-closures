use std::{
    future::Future,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

pub use yield_closures_impl::co;

struct PendOnce(bool);

impl Future for PendOnce {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.get_mut().0 = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub async fn pend_once() {
    PendOnce(false).await
}

pub fn co<F, A, R, T>(f: F) -> impl FnMut(A) -> R
where
    F: FnOnce(mpsc::Receiver<A>, mpsc::SyncSender<R>) -> T,
    T: Future<Output = std::convert::Infallible>,
{
    let (arg_tx, arg_rx) = mpsc::sync_channel(1);
    let (yield_tx, yield_rx) = mpsc::sync_channel(1);
    let mut future = Box::pin(f(arg_rx, yield_tx));
    move |arg| {
        arg_tx.send(arg).unwrap();
        let waker_fn = waker_fn::waker_fn(|| {});
        let mut cx = Context::from_waker(&waker_fn);
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Ready(_) => panic!("logic flaw"),
            Poll::Pending => {}
        }
        yield_rx.recv().unwrap()
    }
}

pub fn co0<F, R, T>(f: F) -> impl FnMut() -> R
where
    F: FnOnce(mpsc::Receiver<()>, mpsc::SyncSender<R>) -> T,
    T: Future<Output = std::convert::Infallible>,
{
    let mut f = co::<F, (), R, T>(f);
    move || f(())
}

pub fn co2<F, A1, A2, R, T>(f: F) -> impl FnMut(A1, A2) -> R
where
    F: FnOnce(mpsc::Receiver<(A1, A2)>, mpsc::SyncSender<R>) -> T,
    T: Future<Output = std::convert::Infallible>,
{
    let mut f = co::<F, (A1, A2), R, T>(f);
    move |a1, a2| f((a1, a2))
}

pub fn co3<F, A1, A2, A3, R, T>(f: F) -> impl FnMut(A1, A2, A3) -> R
where
    F: FnOnce(mpsc::Receiver<(A1, A2, A3)>, mpsc::SyncSender<R>) -> T,
    T: Future<Output = std::convert::Infallible>,
{
    let mut f = co::<F, (A1, A2, A3), R, T>(f);
    move |a1, a2, a3| f((a1, a2, a3))
}

#[doc(hidden)]
#[macro_export]
macro_rules! drop_args {
    () => {{}};
    ($x:ident, ) => {{
        #[allow(clippy::drop_copy)]
        drop($x)
    }};
    ($x0:ident, $x1:ident,) => {{
        #[allow(clippy::drop_copy)]
        drop($x0);
        #[allow(clippy::drop_copy)]
        drop($x1);
    }};
    ($x0:ident, $x1:ident, $x2:ident,) => {{
        #[allow(clippy::drop_copy)]
        drop($x0);
        #[allow(clippy::drop_copy)]
        drop($x1);
        #[allow(clippy::drop_copy)]
        drop($x2);
    }};
    ($x0:ident, $x1:ident, $x2:ident, $x3:ident,) => {{
        #[allow(clippy::drop_copy)]
        drop($x0);
        #[allow(clippy::drop_copy)]
        drop($x1);
        #[allow(clippy::drop_copy)]
        drop($x2);
        #[allow(clippy::drop_copy)]
        drop($x3);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! reassign_args {
    ($rx:ident,) => {{
        let _ = $rx.recv().unwrap();
    }};
    ($rx:ident, $x:ident, ) => {{
        $x = $rx.recv().unwrap();
    }};
    ($rx:ident, $x0:ident, $x1:ident,) => {{
        let a = $rx.recv().unwrap();
        $x0 = a.0;
        $x1 = a.1;
    }};
    ($rx:ident, $x0:ident, $x1:ident, $x2:ident,) => {{
        let a = $rx.recv().unwrap();
        $x0 = a.0;
        $x1 = a.1;
        $x2 = a.2;
    }};
    ($rx:ident, $x0:ident, $x1:ident, $x2:ident, $x3:ident,) => {{
        let a = $rx.recv().unwrap();
        $x0 = a.0;
        $x1 = a.1;
        $x2 = a.2;
        $x3 = a.3;
    }};
}
