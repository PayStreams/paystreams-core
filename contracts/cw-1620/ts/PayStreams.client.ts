/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { InstantiateMsg, ExecuteMsg, Curve, Uint128, Timestamp, Uint64, StreamType, SaturatingLinear, PiecewiseLinear, QueryMsg, Addr, LookupStreamResponse, PaymentStream, CountResponse, StreamsResponse } from "./PayStreams.types";
export interface PayStreamsReadOnlyInterface {
  contractAddress: string;
  lookupStream: ({
    payee,
    payer
  }: {
    payee: string;
    payer: string;
  }) => Promise<LookupStreamResponse>;
  streamCount: () => Promise<CountResponse>;
  streamsByPayee: ({
    limit,
    payee,
    reverse
  }: {
    limit?: number;
    payee: string;
    reverse?: boolean;
  }) => Promise<StreamsResponse>;
  streamsBySender: ({
    limit,
    reverse,
    sender
  }: {
    limit?: number;
    reverse?: boolean;
    sender: string;
  }) => Promise<StreamsResponse>;
  streamsByIndex: ({
    index
  }: {
    index: number;
  }) => Promise<StreamsResponse>;
}
export class PayStreamsQueryClient implements PayStreamsReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.lookupStream = this.lookupStream.bind(this);
    this.streamCount = this.streamCount.bind(this);
    this.streamsByPayee = this.streamsByPayee.bind(this);
    this.streamsBySender = this.streamsBySender.bind(this);
    this.streamsByIndex = this.streamsByIndex.bind(this);
  }

  lookupStream = async ({
    payee,
    payer
  }: {
    payee: string;
    payer: string;
  }): Promise<LookupStreamResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      lookup_stream: {
        payee,
        payer
      }
    });
  };
  streamCount = async (): Promise<CountResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      stream_count: {}
    });
  };
  streamsByPayee = async ({
    limit,
    payee,
    reverse
  }: {
    limit?: number;
    payee: string;
    reverse?: boolean;
  }): Promise<StreamsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      streams_by_payee: {
        limit,
        payee,
        reverse
      }
    });
  };
  streamsBySender = async ({
    limit,
    reverse,
    sender
  }: {
    limit?: number;
    reverse?: boolean;
    sender: string;
  }): Promise<StreamsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      streams_by_sender: {
        limit,
        reverse,
        sender
      }
    });
  };
  streamsByIndex = async ({
    index
  }: {
    index: number;
  }): Promise<StreamsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      streams_by_index: {
        index
      }
    });
  };
}
export interface PayStreamsInterface extends PayStreamsReadOnlyInterface {
  contractAddress: string;
  sender: string;
  createStream: ({
    curve,
    deposit,
    recipient,
    startTime,
    stopTime,
    streamType,
    tokenAddr
  }: {
    curve?: Curve;
    deposit: Uint128;
    recipient: string;
    startTime: Timestamp;
    stopTime: Timestamp;
    streamType?: StreamType;
    tokenAddr: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  withdrawFromStream: ({
    amount,
    denom,
    recipient,
    streamIdx
  }: {
    amount: Uint128;
    denom: string;
    recipient: string;
    streamIdx?: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class PayStreamsClient extends PayStreamsQueryClient implements PayStreamsInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.createStream = this.createStream.bind(this);
    this.withdrawFromStream = this.withdrawFromStream.bind(this);
  }

  createStream = async ({
    curve,
    deposit,
    recipient,
    startTime,
    stopTime,
    streamType,
    tokenAddr
  }: {
    curve?: Curve;
    deposit: Uint128;
    recipient: string;
    startTime: Timestamp;
    stopTime: Timestamp;
    streamType?: StreamType;
    tokenAddr: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_stream: {
        curve,
        deposit,
        recipient,
        start_time: startTime,
        stop_time: stopTime,
        stream_type: streamType,
        token_addr: tokenAddr
      }
    }, fee, memo, _funds);
  };
  withdrawFromStream = async ({
    amount,
    denom,
    recipient,
    streamIdx
  }: {
    amount: Uint128;
    denom: string;
    recipient: string;
    streamIdx?: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      withdraw_from_stream: {
        amount,
        denom,
        recipient,
        stream_idx: streamIdx
      }
    }, fee, memo, _funds);
  };
}