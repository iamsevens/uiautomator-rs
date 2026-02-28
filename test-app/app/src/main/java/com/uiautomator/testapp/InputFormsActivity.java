package com.uiautomator.testapp;

import android.os.Bundle;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.EditText;
import android.widget.Spinner;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;

public class InputFormsActivity extends AppCompatActivity {

    private EditText etUsername;
    private EditText etPassword;
    private EditText etEmail;
    private EditText etComment;
    private Spinner spCountry;
    private TextView tvInputTitle;
    private TextView tvFormResult;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_input_forms);

        etUsername = findViewById(R.id.et_username);
        etPassword = findViewById(R.id.et_password);
        etEmail = findViewById(R.id.et_email);
        etComment = findViewById(R.id.et_comment);
        spCountry = findViewById(R.id.sp_country);
        tvInputTitle = findViewById(R.id.tv_input_title);
        tvFormResult = findViewById(R.id.tv_form_result);

        setupSpinner();
        setupButtons();
    }

    private void setupSpinner() {
        String[] countries = {"USA", "China", "Japan", "Germany", "France", "UK", "Canada"};
        ArrayAdapter<String> adapter = new ArrayAdapter<>(this,
            android.R.layout.simple_spinner_item, countries);
        adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spCountry.setAdapter(adapter);
    }

    private void setupButtons() {
        // Submit button
        findViewById(R.id.btn_submit).setOnClickListener(v -> {
            String username = etUsername.getText().toString();
            String password = etPassword.getText().toString();
            String email = etEmail.getText().toString();
            String comment = etComment.getText().toString();
            String country = spCountry.getSelectedItem().toString();

            if (username.isEmpty() || password.isEmpty() || email.isEmpty()) {
                tvInputTitle.setText("Input Forms - Invalid");
                tvFormResult.setText("Form Result: Please fill all required fields");
            } else {
                String result = String.format(
                    "Form Submitted!\nUser: %s\nEmail: %s\nCountry: %s\nComment: %s",
                    username, email, country, comment.isEmpty() ? "None" : comment
                );
                tvInputTitle.setText("Input Forms - Submitted");
                tvFormResult.setText(result);
            }
        });

        // Clear button
        findViewById(R.id.btn_clear).setOnClickListener(v -> {
            etUsername.setText("");
            etPassword.setText("");
            etEmail.setText("");
            etComment.setText("");
            spCountry.setSelection(0);
            tvInputTitle.setText(getString(R.string.input_title));
            tvFormResult.setText("Form Result: Cleared");
        });
    }
}
