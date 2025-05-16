package main

import (
	"fmt"
	"sync"
)

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
