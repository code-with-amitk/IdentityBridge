* Architecture
  * [Ingestion Tier](Ingestion_Tier.adoc)
  * [Consumer Tier]

# Architecture
- Ingestion Tier(3..200 pods), Consumer Tier(3..50 pods) scaled independently
- Service to scale horizontally. The Rust API is stateless and Tokio-based
```
Ingestion Layer       Kafka              Consumer Layer
---------------     ---------            -------------
100 instances   →   100 partitions   →   50 workers   →   DB
```

```mermaid
flowchart LR
    DNS["DNS<br>Route 53"]
    C1["Collecto1"]
    C2["Collector2"]
    GLB["Global LB<br>Regional LB"]
    
    subgraph Datacenter
        APIGW["API GW<br/>TLS termination"]
        LB["ELB/ALB"]
        subgraph JIMS Server
            subgraph API["Ingestion Tier"]
                KLB["k8s<br>Ingress LB"]
                P1["Pod 1<br/>Tokio Runtime<br> Kafka Producer"]
                P2["Pod 2<br/>Tokio Runtime<br> Kafka Producer"]
                PN["Pod 200<br/>Tokio Runtime<br> Kafka Producer"]
            end
            K[["Kafka<br>Broker<br><br> Partitions<br>P0 P1 P2...P99"]]
            subgraph Workers["Consumer Tier"]
                W1["Worker Pod 1<br><br>→ P0-P9"]
                W2["Worker Pod 2<br><br>→ P10-P19"]
                W3["Worker Pod 3<br><br>→ P20-P29"]
            end
            DB["PostgreSQL"]
        end
    end

    C1 -->|hostname|DNS
    DNS -->|IP|C1

    C1 -->|Batched JSON| GLB
    C2 -->|Batched JSON| GLB
    GLB --> APIGW
    APIGW --> LB
    LB --> KLB
    KLB --> P1
    KLB --> P2
    KLB --> PN

    P1 -->|Topic:<br>identity-events| K
    P2 --> K
    PN --> K

    K -->|consumer group| W1
    K --> W2
    K --> W3

    W1 --> DB
    W2 --> DB
    W3 --> DB
```
