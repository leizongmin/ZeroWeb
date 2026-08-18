package com.leizm.zeroweb;

import android.os.ParcelFileDescriptor;

interface IRoleService {
    void start(in ParcelFileDescriptor socket);
}
