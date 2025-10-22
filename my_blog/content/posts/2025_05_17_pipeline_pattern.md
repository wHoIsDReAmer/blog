+++
title = "파이프라인 패턴"
date = "2025-05-16"
description = "Go로 파이프라인 패턴을 구현해 단계별 처리 흐름을 느슨 결합으로 구성하는 방법을 소개합니다."

[taxonomies]
tags = ["go", "pipeline"]
+++

## 파이프라인 패턴
파이프라인 패턴은 여러 개의 처리 단계로 이루어진 작업 흐름을 만들 때 사용되는 패턴이다.

예를 들어 이벤트를 받아 처리하는 흐름을 생각해보자.
1. 이벤트를 받는다.
2. 이벤트를 처리한다.
3. 이벤트를 커스텀 핸들링한다.

다만, 이벤트를 처리하는 단계에서 여러 개의 처리 단계가 필요할 수 있다. 중간에 API 요청이 있을수도 있고, 데이터베이스 조회가 있을 수도, 어떤 치환이 발생할 수도 있다.

**변화에 유용하지 않은 코드**라면 코드 크기가 커지고, 유지보수성도 떨어지기 마련이다.
심할 경우 책임이 너무 커질수도 있다.

이런 유즈케이스일 경우 파이프라인 패턴을 사용하면 유용하고 동적으로 처리가 가능해진다. 각 스테이지마다 결합이 느슨해지기 때문에 유지보수성도 높아진다. 디버깅도 쉬워지고. 데이터 처리 로직이 복잡하면 사실 안 쓸 이유가 없다.

`Go`로 구현해서 예를 들어보자.
```go
// 파이프라인 정의
// 들어가는 인자가 in, 나오는 인자가 out
type Pipeline[T any] func(<-chan T) <-chan T

// 이벤트를 받아 넘기는 메인 핸들러
type MainHandler struct {
	// 이벤트를 받는 채널
	events chan int
	// 최종 출력값을 위한 내부 쓰기 채널
	internalOut chan int

	// 외부에 노출되는 읽기 전용 최종 출력값 채널
	out <-chan int

	// 파이프라인 목록
	pipes []Pipeline[int]

	wg *sync.WaitGroup
}

func NewMainHandler(wg *sync.WaitGroup) *MainHandler {
	// 기본적으로 양방향 채널로 생성
	eventsChan := make(chan int)
	outChan := make(chan int)

	return &MainHandler{
		events:      eventsChan,
		internalOut: outChan,
		out:         outChan,
		pipes:       []Pipeline[int]{},
		wg:          wg,
	}
}

func (h *MainHandler) AddPipeline(pipe Pipeline[int]) {
	h.pipes = append(h.pipes, pipe)
}

// 엔트리포인트
func (h *MainHandler) Start() {
	h.wg.Add(1)
	go func() {
		defer h.wg.Done()
		defer close(h.internalOut)

		var in <-chan int = h.events
		for _, pipe := range h.pipes {
			in = pipe(in)
		}

		// 모든 파이프라인을 거치고 나서 최종 출력 채널에 쓰기
		for processed := range in {
			h.internalOut <- processed
		}
	}()
}
```
구현체는 아마 이렇게 짜일 것이다.

이제 위 구현체를 이용해서 결과를 관측해보자
```go
func main() {
	var wg sync.WaitGroup

	// 메인 핸들러 생성
	mainHandler := NewMainHandler(&wg)

	// 덧셈 파이프라인 추가
	mainHandler.AddPipeline(func(in <-chan int) <-chan int {
		out := make(chan int)
		go func() {
			defer close(out)
			for event := range in {
				out <- event + 1
			}
		}()
		return out
	})

	// 곱셈 파이프라인 추가
	mainHandler.AddPipeline(func(in <-chan int) <-chan int {
		out := make(chan int)
		go func() {
			defer close(out)
			for event := range in {
				out <- event * 2
			}
		}()
		return out
	})

	// 메인 핸들러 시작
	mainHandler.Start()

	go func() {
		for out := range mainHandler.out {
			fmt.Println(out)
		}
	}()

	// 이벤트 전송
	// (1+1)*2 = 4
	// (2+1)*2 = 6
	// (3+1)*2 = 8
	mainHandler.events <- 1
	mainHandler.events <- 2
	mainHandler.events <- 3

	close(mainHandler.events)

	// 모든 고루틴이 종료될 때까지 대기
	wg.Wait()
}
```

이렇게 파이프라인 패턴을 구현할 수 있다. 현재 예제는 단순한 연산만 수행하지만, 실제 프로덕션 환경에서는 각 스테이지가 복잡한 비즈니스 로직을 처리할 수 있다. 이럴 때 파이프라인 패턴의 장점이 드러날 것이다. 각 스테이지가 독립적으로 동작하기 때문에 디버깅이 용이하고, 성능 최적화도 단계별로 진행할 수 있기 때문이다.
