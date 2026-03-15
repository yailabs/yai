// SPDX-License-Identifier: GPL-2.0
/*
 * Copyright (c) 2018, 2019 Oracle. All rights reserved.
 */

#include <yai/sunrpc/clnt.h>
#include <yai/sunrpc/sched.h>
#include <yai/sunrpc/svc.h>
#include <yai/sunrpc/svc_xprt.h>
#include <yai/sunrpc/auth_gss.h>
#include <yai/sunrpc/gss_err.h>

#define CREATE_TRACE_POINTS
#include <trace/events/rpcgss.h>
