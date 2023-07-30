use std::sync::mpsc;
use std::thread;

pub trait Task {
    type Output: Send;
    fn run(&self) -> Option<Self::Output>;
}

pub struct WorkQueue<TaskType: 'static + Task + Send> {
    send_tasks: Option<spmc::Sender<TaskType>>, // Option because it will be set to None to close the queue
    recv_tasks: spmc::Receiver<TaskType>,
    //send_output: mpsc::Sender<TaskType::Output>, // not need in the struct: each worker will have its own clone.
    recv_output: mpsc::Receiver<TaskType::Output>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<TaskType: 'static + Task + Send> WorkQueue<TaskType> {
    pub fn new(n_workers: usize) -> WorkQueue<TaskType> {
        // create the channels; start the worker threads; record their JoinHandles
        
        // channels created for jobs going into the queue and the results coming out (work queue doesn't distinguish between results from the tasks
        let (send_tasks, recv_tasks) = spmc::channel();
        let (mpsc_sender, recv_output) = mpsc::channel();

        let mut workers = Vec::<thread::JoinHandle<()>>::new();
        for n in 0..n_workers {
            let snd = mpsc_sender.clone();
            let rcv = recv_tasks.clone();
            workers.push(thread::spawn( move || {
                Self::run(rcv, snd);
            }));
        }

        WorkQueue::<TaskType> {
            send_tasks: Some(send_tasks),
            recv_tasks,
            recv_output,
            workers
        }
        
    }

    fn run(recv_tasks: spmc::Receiver<TaskType>, send_output: mpsc::Sender<TaskType::Output>) {
        // TODO: the main logic for a worker thread
        loop {
            let task_result = recv_tasks.recv();
            // task_result will be Err() if the spmc::Sender has been destroyed and no more messages can be received here]
            match task_result {
                Err(e) => {
                    //println!("shutting down");
                    return;
                }
                Ok(r) => {
                    //println!("received correctly");

                    let output = r.run();

                    match output {
                        Some(x) => {
                            //println!("solution found");
                            let _ = send_output.send(x);
                        }
                        None => {
                            println!("no solution");
                            todo!()
                        }
                    }
                }
            }
        }
    }

    pub fn enqueue(&mut self, t: TaskType) -> Result<(), spmc::SendError<TaskType>> {
        // send this task to a worker
        match &mut self.send_tasks {
            Some(snd) => {
                snd.send(t)?;
                Ok(())
            }
            None => {
                Ok(())
            }
        }
    }

    // Helper methods that let you receive results in various ways
    pub fn iter(&mut self) -> mpsc::Iter<TaskType::Output> {
        self.recv_output.iter()
    }
    pub fn recv(&mut self) -> TaskType::Output {
        self.recv_output
            .recv()
            .expect("I have been shutdown incorrectly")
    }
    pub fn try_recv(&mut self) -> Result<TaskType::Output, mpsc::TryRecvError> {
        self.recv_output.try_recv()
    }
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<TaskType::Output, mpsc::RecvTimeoutError> {
        self.recv_output.recv_timeout(timeout)
    }

    pub fn shutdown(&mut self) {
        // Destroy the spmc::Sender so everybody knows no more tasks are incoming;
        // drain any pending tasks in the queue; wait for each worker thread to finish.
        // HINT: Vec.drain(..)
        // std::mem::replace(&mut self.send_tasks, None);
        self.send_tasks = None;

        loop {
            match self.recv_tasks.recv() {
                Ok(_) => {
                    //do nothing, discard
                }
                Err(_) => {
                    break
                }
            }
        }
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

impl<TaskType: 'static + Task + Send> Drop for WorkQueue<TaskType> {
    fn drop(&mut self) {
        // "Finalisation in destructors" pattern: https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
        match self.send_tasks {
            None => {} // already shut down
            Some(_) => self.shutdown(),
        }
    }
}
