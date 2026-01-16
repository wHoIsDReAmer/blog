+++
title = "Consensus Engine (합의 엔진) 이해하기"
date = "2026-01-15"
description = "Consensus Engine을 이해해봅시다."

[taxonomies]
tags = ["blockchain", "crypto"]
+++

근래에 블록체인에 꽤 관심이 생겼다. 생태계를 쭉 둘러봤는데, 대략적으로 dApp이나 Consensus 엔진 정도가 눈에 먼저 보였던 것 같다.

차차 전반적으로 이해한 걸 바탕으로 글을 쓸 예정이다. 다만 우선 첫글은 가장 low-level 레이어인 합의 엔진 (Consensus Engine)을 이해해보려고 한다.

이론도 얼추 까보고, 실제 구현체도 까보면서 이해해보자.

블록체인에서 가장 low-level 단을 책임지고 있는 것이 `Consensus` 엔진이다.

이런 게 왜 필요할까? 이걸 알기 위해선 우선 분산 시스템을 이해할 필요가 있다.

## 분산 시스템

