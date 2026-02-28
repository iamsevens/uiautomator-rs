package com.uiautomator.testapp;

import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.TextView;
import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.app.AppCompatActivity;
import com.google.android.material.bottomsheet.BottomSheetBehavior;
import com.google.android.material.bottomsheet.BottomSheetDialog;

public class DialogsActivity extends AppCompatActivity {

    private TextView tvDialogResult;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_dialogs);

        tvDialogResult = findViewById(R.id.tv_dialog_result);

        setupButtons();
    }

    private void setupButtons() {
        // Alert Dialog
        findViewById(R.id.btn_alert).setOnClickListener(v -> {
            new AlertDialog.Builder(this)
                .setTitle("Alert Dialog")
                .setMessage("This is an alert dialog message.")
                .setPositiveButton("OK", (dialog, which) -> {
                    tvDialogResult.setText("Dialog Result: Alert OK clicked");
                })
                .show();
        });

        // Confirm Dialog
        findViewById(R.id.btn_confirm).setOnClickListener(v -> {
            new AlertDialog.Builder(this)
                .setTitle("Confirm Dialog")
                .setMessage("Do you want to proceed?")
                .setPositiveButton("Yes", (dialog, which) -> {
                    tvDialogResult.setText("Dialog Result: Confirmed YES");
                })
                .setNegativeButton("No", (dialog, which) -> {
                    tvDialogResult.setText("Dialog Result: Confirmed NO");
                })
                .show();
        });

        // Custom Dialog
        findViewById(R.id.btn_custom).setOnClickListener(v -> {
            View customView = LayoutInflater.from(this).inflate(R.layout.dialog_custom, null);
            AlertDialog dialog = new AlertDialog.Builder(this)
                .setView(customView)
                .create();

            customView.findViewById(R.id.btn_dialog_ok).setOnClickListener(view -> {
                tvDialogResult.setText("Dialog Result: Custom dialog OK");
                dialog.dismiss();
            });

            customView.findViewById(R.id.btn_dialog_cancel).setOnClickListener(view -> {
                tvDialogResult.setText("Dialog Result: Custom dialog Cancel");
                dialog.dismiss();
            });

            dialog.show();
        });

        // Bottom Sheet
        findViewById(R.id.btn_bottom_sheet).setOnClickListener(v -> {
            BottomSheetDialog bottomSheet = new BottomSheetDialog(this);
            View sheetView = LayoutInflater.from(this).inflate(R.layout.bottom_sheet, null);

            sheetView.findViewById(R.id.btn_sheet_option1).setOnClickListener(view -> {
                tvDialogResult.setText("Dialog Result: Bottom Sheet Option 1");
                bottomSheet.dismiss();
            });

            sheetView.findViewById(R.id.btn_sheet_option2).setOnClickListener(view -> {
                tvDialogResult.setText("Dialog Result: Bottom Sheet Option 2");
                bottomSheet.dismiss();
            });

            sheetView.findViewById(R.id.btn_sheet_option3).setOnClickListener(view -> {
                tvDialogResult.setText("Dialog Result: Bottom Sheet Option 3");
                bottomSheet.dismiss();
            });

            bottomSheet.setContentView(sheetView);
            bottomSheet.show();

            // 使用 post 延迟确保 Bottom Sheet 完全展开
            sheetView.post(() -> {
                View parent = (View) sheetView.getParent();
                if (parent != null) {
                    BottomSheetBehavior<View> behavior = BottomSheetBehavior.from(parent);
                    behavior.setState(BottomSheetBehavior.STATE_EXPANDED);
                    behavior.setPeekHeight(0);
                    behavior.setSkipCollapsed(true);
                }
            });
        });
    }
}
