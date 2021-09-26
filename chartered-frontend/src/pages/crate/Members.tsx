import React = require("react");
import { useState } from "react";
import {
  PersonPlus,
  Trash,
  CheckLg,
  Save,
  PlusLg,
} from "react-bootstrap-icons";
import { authenticatedEndpoint, RoundedPicture, RoundedPicture, useAuthenticatedRequest } from "../../util";
import { useAuth } from "../../useAuth";
import { Button, Modal } from "react-bootstrap";
import { AsyncTypeahead } from "react-bootstrap-typeahead";
import { debounce } from "lodash";
import _ = require("lodash");
import ReactPlaceholder from "react-placeholder";

interface Member {
  uuid: string;
  permissions?: string[];
  username: string;
}

export default function Members({
  members,
  possiblePermissions,
  saveMemberPermissions,
  deleteMember,
}: {
  members: Member[];
  possiblePermissions?: string[];
  saveMemberPermissions: (
    prospectiveMember: boolean,
    uuid: string,
    selectedPermissions: string[]
  ) => Promise<any>;
  deleteMember: (uuid: string) => Promise<any>;
}) {
  const [prospectiveMembers, setProspectiveMembers] = useState([]);

  React.useEffect(() => {
    setProspectiveMembers(
      prospectiveMembers.filter((prospectiveMember) => {
        _.findIndex(
          members,
          (member) => member.uuid === prospectiveMember.uuid
        ) === -1;
      })
    );
  }, [members]);

  return (
    <table className="table table-striped">
      <tbody>
        {members.map((member, index) => (
          <MemberListItem
            key={index}
            member={member}
            prospectiveMember={false}
            possiblePermissions={possiblePermissions}
            saveMemberPermissions={saveMemberPermissions}
            deleteMember={deleteMember}
          />
        ))}

        {prospectiveMembers.map((member, index) => (
          <MemberListItem
            key={index}
            member={member}
            prospectiveMember={true}
            possiblePermissions={possiblePermissions}
            saveMemberPermissions={saveMemberPermissions}
            deleteMember={deleteMember}
          />
        ))}

        {possiblePermissions ? (
          <MemberListInserter
            onInsert={(username, userUuid) =>
              setProspectiveMembers([
                ...prospectiveMembers,
                {
                  uuid: userUuid,
                  username,
                  permissions: ["VISIBLE"],
                },
              ])
            }
            existingMembers={members}
          />
        ) : (
          <></>
        )}
      </tbody>
    </table>
  );
}

function MemberListItem({
  member,
  prospectiveMember,
  possiblePermissions,
  saveMemberPermissions,
  deleteMember,
}: {
  member: Member;
  prospectiveMember: boolean;
  possiblePermissions?: string[];
  saveMemberPermissions: (
    prospectiveMember: boolean,
    uuid: string,
    selectedPermissions: string[]
  ) => Promise<any>;
  deleteMember: (uuid: string) => Promise<any>;
}) {
  const [selectedPermissions, setSelectedPermissions] = useState(
    member.permissions
  );
  const [deleting, setDeleting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);

  let itemAction = <></>;

  const doSaveMemberPermissions = async () => {
    setSaving(true);

    try {
      await saveMemberPermissions(
        prospectiveMember,
        member.uuid,
        selectedPermissions
      );
    } catch (e) {
      setError(error);
    } finally {
      setSaving(false);
    }
  };

  const doDelete = async () => {
    setSaving(true);

    try {
      deleteMember(member.uuid);
    } catch (e) {
      setError(error);
    } finally {
      setSaving(false);
    }
  };

  if (!possiblePermissions) {
    // the current user can't perform any actions
  } else if (saving) {
    itemAction = (
      <button type="button" className="btn">
        <div
          className="spinner-grow spinner-grow-sm text-primary"
          role="status"
        >
          <span className="visually-hidden">Loading...</span>
        </div>
      </button>
    );
  } else if (
    !prospectiveMember &&
    selectedPermissions.indexOf("VISIBLE") === -1
  ) {
    itemAction = (
      <button
        type="button"
        className="btn text-danger"
        onClick={() => setDeleting(true)}
      >
        <Trash />
      </button>
    );
  } else if (
    prospectiveMember ||
    selectedPermissions.sort().join(",") != member.permissions.sort().join(",")
  ) {
    itemAction = (
      <button
        type="button"
        className="btn text-success"
        onClick={doSaveMemberPermissions}
      >
        <CheckLg />
      </button>
    );
  }

  return (
    <>
      <DeleteModal
        show={deleting === true}
        onCancel={() => setDeleting(false)}
        onConfirm={() => doDelete()}
        username={member.username}
      />

      <ErrorModal error={error} onClose={() => setError(null)} />

      <tr>
        <td className="align-middle fit">
          <RoundedPicture src="http://placekitten.com/48/48" height="48px" width="48px" />
        </td>

        <td className="align-middle">
          <strong>{member.username}</strong>
          {/*<br />
          <em>(that's you!)</em>*/}
        </td>

        {possiblePermissions && member.permissions ? (
          <>
            <td className="align-middle">
              <RenderPermissions
                possiblePermissions={possiblePermissions}
                selectedPermissions={selectedPermissions}
                userUuid={member.uuid}
                onChange={setSelectedPermissions}
              />
            </td>

            <td className="align-middle fit">{itemAction}</td>
          </>
        ) : (
          <></>
        )}
      </tr>
    </>
  );
}

function MemberListInserter({
  onInsert,
  existingMembers,
}: {
  existingMembers: Member[];
  onInsert: (username, user_uuid) => any;
}) {
  const auth = useAuth();
  const searchRef = React.useRef(null);
  const [loading, setLoading] = useState(false);
  const [options, setOptions] = useState([]);
  const [error, setError] = useState("");

  const handleSearch = async (query) => {
    setLoading(true);
    setError("");

    try {
      let res = await fetch(
        authenticatedEndpoint(
          auth,
          `users/search?q=${encodeURIComponent(query)}`
        )
      );
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      }

      setOptions(json.users || []);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (selected) => {
    onInsert(selected[0].username, selected[0].user_uuid);
    searchRef.current.clear();
  };

  return (
    <tr>
      <td className="align-middle fit">
        <div
          className="d-flex align-items-center justify-content-center rounded-circle"
          style={{
            width: "48px",
            height: "48px",
            background: "#DEDEDE",
            fontSize: "1rem",
          }}
        >
          <PersonPlus />
        </div>
      </td>

      <td className="align-middle">
        <AsyncTypeahead
          id="search-new-user"
          onSearch={handleSearch}
          filterBy={(option) =>
            _.findIndex(
              existingMembers,
              (existing) => option.user_uuid === existing.uuid
            ) === -1
          }
          labelKey="username"
          options={options}
          isLoading={loading}
          placeholder="Search for User"
          onChange={handleChange}
          ref={searchRef}
          renderMenuItemChildren={(option, props) => (
            <>
              <RoundedPicture
                src="http://placekitten.com/24/24"
                height="24px"
                width="24px"
                className="me-2"
              />
              <span>{option.username}</span>
            </>
          )}
        />

        <div className="text-danger">{error}</div>
      </td>

      <td className="align-middle"></td>

      <td className="align-middle">
        <button type="button" className="btn text-dark pe-none">
          <PlusLg />
        </button>
      </td>
    </tr>
  );
}

function RenderPermissions({
  possiblePermissions,
  selectedPermissions,
  userUuid,
  onChange,
}: {
  possiblePermissions: string[];
  selectedPermissions: string[];
  userUuid: string;
  onChange: (permissions) => any;
}) {
  return (
    <div className="grid" style={{ "--bs-gap": 0 }}>
      {possiblePermissions.map((permission) => (
        <div
          key={permission + userUuid}
          className="form-check g-col-12 g-col-md-4"
        >
          <input
            className="form-check-input"
            type="checkbox"
            value="1"
            id={`checkbox-${userUuid}-${permission}`}
            checked={selectedPermissions.indexOf(permission) > -1}
            onChange={(e) => {
              let newUserPermissions = new Set(selectedPermissions);

              if (e.target.checked) {
                newUserPermissions.add(permission);
              } else {
                newUserPermissions.delete(permission);
              }

              onChange(Array.from(newUserPermissions));
            }}
          />
          <label
            className="form-check-label"
            htmlFor={`checkbox-${userUuid}-${permission}`}
          >
            {permission}
          </label>
        </div>
      ))}
    </div>
  );
}

function DeleteModal(props: {
  show: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  username: string;
}) {
  return (
    <Modal
      show={props.show}
      onHide={props.onCancel}
      size="lg"
      aria-labelledby="delete-modal-title"
      centered
    >
      <Modal.Header closeButton>
        <Modal.Title id="delete-modal-title">
          Are you sure you wish to remove this member from the crate?
        </Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>
          Are you sure you wish to remove <strong>{props.username}</strong> from
          the crate?
        </p>
      </Modal.Body>
      <Modal.Footer>
        <Button onClick={props.onCancel} variant="primary">
          Close
        </Button>
        <Button onClick={props.onConfirm} variant="danger">
          Delete
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

function ErrorModal(props: { error?: string; onClose: () => void }) {
  return (
    <Modal
      show={props.error != null}
      onHide={props.onClose}
      size="lg"
      aria-labelledby="error-modal-title"
      centered
    >
      <Modal.Header closeButton>
        <Modal.Title id="error-modal-title">Error</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>{props.error}</p>
      </Modal.Body>
      <Modal.Footer>
        <Button onClick={props.onClose} variant="primary">
          Close
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
