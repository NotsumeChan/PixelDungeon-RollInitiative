package io.github.notsumechan.pdri;

import android.app.NativeActivity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

/** NativeActivity con el selector de archivos para importar mods (app/src/mods.rs). */
public class MainActivity extends NativeActivity {
    private static final int PICK_ZIP = 1;
    private static final long MAX = 64L << 20;

    /** Lo llama Rust (JNI): abre el selector de documentos del sistema para elegir un zip. */
    public void pickZip() {
        runOnUiThread(() -> {
            Intent i = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            i.addCategory(Intent.CATEGORY_OPENABLE);
            i.setType("*/*");
            i.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {"application/zip", "application/x-zip-compressed", "application/octet-stream"});
            startActivityForResult(i, PICK_ZIP);
        });
    }

    /** Copia el zip elegido a files/mod_import.zip (vía .part: Rust lo ve sólo completo) y su nombre a mod_import.name. */
    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != PICK_ZIP || result != RESULT_OK || data == null || data.getData() == null) return;
        Uri uri = data.getData();
        new Thread(() -> {
            File dir = getFilesDir();
            File part = new File(dir, "mod_import.part");
            String name = "mod";
            try (Cursor c = getContentResolver().query(uri, new String[] {OpenableColumns.DISPLAY_NAME}, null, null, null)) {
                if (c != null && c.moveToFirst()) name = c.getString(0);
            } catch (Exception e) {
                // sin nombre: "mod"
            }
            try (InputStream in = getContentResolver().openInputStream(uri); OutputStream out = new FileOutputStream(part)) {
                byte[] b = new byte[65536];
                long total = 0;
                int n;
                while ((n = in.read(b)) > 0) {
                    total += n;
                    if (total > MAX) throw new java.io.IOException("too big");
                    out.write(b, 0, n);
                }
                try (OutputStream o = new FileOutputStream(new File(dir, "mod_import.name"))) {
                    o.write(name.getBytes("UTF-8"));
                }
            } catch (Exception e) {
                part.delete();
                return;
            }
            part.renameTo(new File(dir, "mod_import.zip"));
        }).start();
    }
}
