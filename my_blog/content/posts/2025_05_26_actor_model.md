+++
title = "액터 모델"
date = "2025-05-26"
description = "액터 모델의 핵심 원리와 장단점, 메시지 기반 동시성을 Rust 예제로 살펴봅니다."

[taxonomies]
tags = ["actor", "asynchronous", "concurrency"]
+++

## 액터 모델

1973년 Carl Hewitt가 처음 제안한 모델로, 분산 시스템과 동시성 프로그래밍의 복잡성을 해결하기 위해 고안되었다.

액터 모델은 프로세스 간 통신을 추상화하여 동시성 프로그래밍을 단순화하는 모델이다.

`모든 것은 actor다` 라는 철학을 바탕으로 나왔으며, `actor`마다 내부적으로 메시지를 주고받는 방식으로 동작한다. 메시지를 주고받는 방식으로 동작하기 때문에 메시지 큐를 사용하는 방식과 유사하다.

하나씩 톺아보자.

### 핵심 원리

액터 모델의 기본 아이디어는 다음과 같다:

- **액터(Actor)**: 계산의 기본 단위로, 각자 독립된 상태와 행동을 가진다
- **메시지 전달**: 액터 간의 유일한 통신 수단으로, 비동기적으로 이루어진다
- **불변성**: 메시지는 불변이며, 액터의 상태는 외부에서 직접 접근할 수 없다

### 특징

- **상태 캡슐화**: 각 actor는 자신의 상태를 캡슐화하고 있으며, 외부에서 직접 접근할 수 없다.
- **메시지 기반 통신**: actor는 메시지를 통해서만 서로 통신한다.
- **위치 투명성**: actor는 로컬 또는 원격 위치를 구분하지 않고 메시지를 주고받는다.
- **비동기 처리**: 메시지 전송은 논블로킹이며, 응답을 기다리지 않는다.
- **독립적 실행**: 각 actor는 독립적으로 실행되며, 다른 actor의 실행에 영향을 받지 않는다.

### 액터의 행동

액터가 메시지를 받았을 때 할 수 있는 일은 다음 세 가지뿐이다:

1. **새로운 액터 생성**: 다른 액터를 생성할 수 있다
2. **메시지 전송**: 다른 액터에게 메시지를 보낼 수 있다
3. **상태 변경**: 다음 메시지를 위해 자신의 상태를 변경할 수 있다

### 장점

- **데드락 방지**: 공유 상태가 없어 데드락이 발생하지 않는다
- **확장성**: 액터 수를 쉽게 늘려 시스템을 확장할 수 있다
- **오류 격리**: 한 액터의 실패가 다른 액터에 영향을 주지 않는다
- **분산 처리**: 네트워크를 통해 분산된 액터들이 투명하게 통신할 수 있다
- **테스트 용이성**: 각 액터를 독립적으로 테스트할 수 있다

### 단점

- **디버깅 어려움**: 비동기 메시지 전달로 인해 콜스택 추적이 어렵다
- **메시지 순서**: 메시지 전달 순서가 보장되지 않을 수 있다
- **메모리 오버헤드**: 각 액터마다 메일박스를 유지해야 한다

### 실제 사용 사례

- **Erlang/OTP**: 통신 시스템에서 수백만 개의 액터를 동시에 실행
- **Akka**: 대용량 트래픽을 처리하는 웹 서비스 (LinkedIn, Twitter 등)
- **Orleans**: Microsoft의 분산 게임 서버 플랫폼
- **CAF**: C++ 액터 프레임워크로 고성능 시스템 구축

---

사실 설명만 들으면 직접적으로 와닿지 않는다. 결국 최상위 추상화 레벨에서의 설명이기 때문이다. 실제로 구현해보면 그렇게 어렵지는 않다.

또한, 뮤텍스를 안쓴다고는 하는데, 내부 구현에서는 뮤텍스를 사용한다 ㅋㅋ

단순히 락의 책임이 개발자에서 액터 모델로 넘어간 거 뿐이다.

그리고 구현할 때 애플리케이션 레이어에서 구현할텐데, 실제로는 이런식으로 쓰이는 디자인보다는 실제 노드 자체를 액터로 취급하고 노드 간 통신을 액터로 취급하는 경우가 더 많다. 그러니까, 분산 시스템에서 더 많이 채택되는 디자인이다.

우선 구현해보기 전 사진부터 보자.
![actor_model](https://labviewwiki.org/w/images/f/f7/Actor_Framework_Communication.png)

### Rust로 구현한 액터 모델

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum CalculatorMessage {
    Add { a: f64, b: f64, reply: Sender<f64> },
    Multiply { a: f64, b: f64, reply: Sender<f64> },
    Divide { a: f64, b: f64, reply: Sender<Result<f64, String>> },
    GetHistory { reply: Sender<Vec<String>> },
    Clear,
    Stop,
}

struct CalculatorActor {
    receiver: Receiver<CalculatorMessage>,
    history: Vec<String>,
}

impl CalculatorActor {
    fn new(receiver: Receiver<CalculatorMessage>) -> Self {
        CalculatorActor {
            receiver,
            history: Vec::new(),
        }
    }

    fn run(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            match message {
                CalculatorMessage::Add { a, b, reply } => {
                    let result = a + b;
                    self.history.push(format!("{} + {} = {}", a, b, result));
                    let _ = reply.send(result);
                }
                CalculatorMessage::Multiply { a, b, reply } => {
                    let result = a * b;
                    self.history.push(format!("{} * {} = {}", a, b, result));
                    let _ = reply.send(result);
                }
                CalculatorMessage::Divide { a, b, reply } => {
                    if b == 0.0 {
                        let _ = reply.send(Err("Division by zero".to_string()));
                    } else {
                        let result = a / b;
                        self.history.push(format!("{} / {} = {}", a, b, result));
                        let _ = reply.send(Ok(result));
                    }
                }
                CalculatorMessage::GetHistory { reply } => {
                    let _ = reply.send(self.history.clone());
                }
                CalculatorMessage::Clear => {
                    self.history.clear();
                    println!("History cleared");
                }
                CalculatorMessage::Stop => {
                    println!("Calculator stopping...");
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
struct CalculatorHandle {
    sender: Sender<CalculatorMessage>,
}

impl CalculatorHandle {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let mut actor = CalculatorActor::new(receiver);

        thread::spawn(move || {
            actor.run();
        });

        CalculatorHandle { sender }
    }

    fn add(&self, a: f64, b: f64) -> f64 {
        let (reply_sender, reply_receiver) = mpsc::channel();
        let _ = self.sender.send(CalculatorMessage::Add { a, b, reply: reply_sender });
        reply_receiver.recv().unwrap_or(0.0)
    }

    fn multiply(&self, a: f64, b: f64) -> f64 {
        let (reply_sender, reply_receiver) = mpsc::channel();
        let _ = self.sender.send(CalculatorMessage::Multiply { a, b, reply: reply_sender });
        reply_receiver.recv().unwrap_or(0.0)
    }

    fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        let _ = self.sender.send(CalculatorMessage::Divide { a, b, reply: reply_sender });
        reply_receiver.recv().unwrap_or(Err("Communication error".to_string()))
    }

    fn get_history(&self) -> Vec<String> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        let _ = self.sender.send(CalculatorMessage::GetHistory { reply: reply_sender });
        reply_receiver.recv().unwrap_or_default()
    }

    fn clear(&self) {
        let _ = self.sender.send(CalculatorMessage::Clear);
    }

    fn stop(&self) {
        let _ = self.sender.send(CalculatorMessage::Stop);
    }
}
```

분산 시스템이나 실시간 시스템, 높은 동시성이 필요한 시스템에서 좋게 써먹을 수 있다. 성능적으로도 그렇고 디자인적으로도 우수하다.

## 참조

- [Actor Model](https://en.wikipedia.org/wiki/Actor_model)
