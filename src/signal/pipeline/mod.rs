pub(crate) mod cgroup_handler;

pub(crate) use cgroup_handler::CGroupHandler;

pub(crate) trait Handler<T>
where
    T: prost::Message + 'static,
{
    fn handle(&self, msg: T) -> T;
}

#[derive(Default)]
pub(crate) struct Pipeline<T>
where
    T: prost::Message + 'static,
{
    handlers: Vec<Box<dyn Handler<T>>>,
}

unsafe impl<T> Send for Pipeline<T> where T: prost::Message + 'static {}

impl<T> Pipeline<T>
where
    T: prost::Message + 'static,
{
    pub(crate) fn add_handler(&mut self, handler: Box<dyn Handler<T>>) {
        self.handlers.push(handler);
    }

    pub(crate) fn process(&self, msg: T) -> T {
        self.handlers.iter().fold(msg, |msg, cmd| cmd.handle(msg))
    }
}

#[cfg(test)]
mod test {
    use crate::signal::signal_proto::LwTask;
    #[test]
    fn test_streamline() {
        struct h1 {}

        impl super::Handler<LwTask> for h1 {
            fn handle(&self, mut msg: LwTask) -> LwTask {
                msg.boot_ns = 12345;
                msg
            }
        }

        struct h2 {}

        impl super::Handler<LwTask> for h2 {
            fn handle(&self, mut msg: LwTask) -> LwTask {
                msg.session_id = 54321;
                msg
            }
        }

        let mut streamline = super::Pipeline::<LwTask>::default();
        streamline.add_handler(Box::new(h1 {}));
        streamline.add_handler(Box::new(h2 {}));

        let task = streamline.process(LwTask::default());
        assert_eq!(task.session_id, 54321);
        assert_eq!(task.boot_ns, 12345);
    }
}
