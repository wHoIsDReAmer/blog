+++
title = "머클 트리 (Merkle Tree)"
date = "2026-05-11"
aliases = ["posts/2026-05-11-merkle-tree"]
description = "블록체인에서 머클 트리란?"

[taxonomies]
tags = ["blockchain", "crypto"]
+++

블록체인 L1 레이어에선 블록이 정상적인지 판별하기 위해 머클 트리를 이용해서 만든 트랜잭션 해시값을 대조한다.

![merkle-2](../../images/merkle-2.png)

위와 같이 각 트랜잭션들의 해시를 트리 형태로 합해서(H(L1 | L2)) 머클 루트를 만든다. 해시 특성상 1 바이트라도 바뀌면 트리 전체가 무너지기 때문에 정합성을 검증할 수 있다. 머클 트리 자체의 방식은 이게 끝이다. 설명할게 따로 없다. 그냥 해시 이어붙여서 또 해시하기, 그걸 올리기다.

중요한 부분은 이게 실제 블록체인에서 어떻게 사용되는지이다. 보통 일반적으로 블록의 형태는 다음과 같다.

```rust
type Hash = [u8; 32];

struct BlockHeader {
    version: u32,
    prev_block_hash: Hash,
    merkle_root: Hash,
    timestamp: u64,
}

struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
}
```

## 블록체인이 머클 루트를 쓰는 시점

크게 세 군데가 있는데,

1. 블록 생성: 블록 생성자(PoW면 마이너, PoS면 프로포저)는 멤풀에서 트랜잭션을 골라 머클 트리를 계산하고, 그 루트를 `merkle_root`에 박는다. 이 시점부터 본문은 사실상 동결이다. 트랜잭션 하나만 바꿔도 루트가 바뀌고, 그 위에서 진행한 봉인 작업(PoW 해싱이든 PoS 서명이든)이 전부 무효가 된다.

2. 블록 검증: 풀노드는 본문 트랜잭션 전부로 머클 루트를 재계산해서 헤더의 `merkle_root`와 대조한다. 일치하지 않으면 거부 처리된다. 트랜잭션 위변조나 누락은 여기서 잡힌다.

3. 풀노드 없이 검증할 때 (라이트 클라이언트): 32바이트 해시 하나가 본문 전체를 대표하니까, 헤더만 따라가도 체인 상태를 검증할 수 있다. 본문(MB 단위)은 필요할 때만 받는다. 모바일 지갑이 이 방식으로 동작한다.

그럼 라이트 클라이언트는 내 트랜잭션이 이 블록에 진짜 들어있다는 걸 어떻게 검증할까. 여기서 머클 증명이 나온다.

## 머클 증명 (Merkle Proof)

트리에서 잎부터 루트까지 올라가는 경로의 형제(sibling) 해시만 있으면 된다. 트랜잭션이 N개일 때 필요한 해시는 log₂(N)개이다.

Tx1~Tx4 네 트랜잭션이 잎으로 있는 트리(`L1 = hash(H1|H2)`, `L2 = hash(H3|H4)`, `root = hash(L1|L2)`)에서 Tx3이 들어있다는 걸 증명한다고 치자.

- 클라이언특 가지고 있는 것: Tx3
- 밸리데이터가 줘야 할 것: H4, L1
- 검증:
  1. H3 = hash(Tx3)
  2. L2' = hash(H3 | H4)
  3. root' = hash(L1 | L2')
  4. root'가 헤더의 `merkle_root`와 같으면 끝

이게 끝이다. 별 거 없어 보이지만 이 검증 단계 개수에 머클 증명의 효율성이 다 들어있다.

트리 구조는 한 층 올라갈 때마다 노드 수가 절반이 되니까, 잎이 N개라면 루트까지 깊이는 log₂(N)이고, 각 층에서 형제 해시 하나씩만 받으면 위로 합쳐 올라갈 수 있다. 필요한 해시 개수도 결국 log₂(N)개가 된다.

숫자로 보면 얼추 감이 잡히는데,

- 트랜잭션 4개 → 해시 **2개**
- 트랜잭션 1,024개 → 해시 **10개**
- 트랜잭션 100만 개 → 해시 **20개**
- 트랜잭션 10억 개 → 해시 **30개**

가 된다. 본문은 O(N)으로 늘어나는데 증명은 O(log N)로만 자란다. 트랜잭션을 1000배로 늘려도 해시는 10개 더 받으면 끝인 것이다.

```rust
enum Direction {
    Left,
    Right,
}

struct MerkleProof {
    leaf: Hash,
    siblings: Vec<(Hash, Direction)>,
}

fn verify(proof: &MerkleProof, root: &Hash) -> bool {
    let mut acc = proof.leaf;
    for (sibling, dir) in &proof.siblings {
        acc = match dir {
            Direction::Left  => hash_pair(sibling, &acc),
            Direction::Right => hash_pair(&acc, sibling),
        };
    }
    &acc == root
}
```

방향이 필요한 이유는 `hash(A|B) ≠ hash(B|A)`이기 때문이다. (문자열을 앞뒤로 단순히 붙이기 때문) 형제가 왼쪽인지 오른쪽인지 모르면 다른 루트가 나온다.

이 구조의 안전성은 결국 해시 함수의 충돌 저항성에 기대는데 공격자가 `hash(X | H4) = hash(H3 | H4)`인 가짜 `X`를 만들 수 있다면 변조가 가능하겠지만.. SHA-256 같은 함수에선 확률적으로 불가능하다. 

## 다른 곳에서도 쓰인다

해시 트리 자체는 블록체인 전용 자료구조가 아니라서 곳곳에 박혀있다.

- Git: 객체 저장소가 머클 DAG다. 커밋 해시 하나가 트리 전체를 대표한다.
- Certificate Transparency: TLS 인증서 발급 로그
- 이더리움: 상태(state)까지 머클로 박는 Merkle Patricia Trie를 쓴다
- ZK 시스템: 상태 커밋먼트로 머클 루트 사용

내부 구조 까보면 해시 묶어서 트리 쌓고 루트만 들고 다니기가 전부인데 의외로 응용 폭이 넓은 자료구조다.
