import { useState, useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { Trash, CheckLg, PlusLg, PersonPlusFill } from "react-bootstrap-icons";
import { authenticatedEndpoint, ProfilePicture } from "../../util";
import { useAuth } from "../../useAuth";
import { Button, Modal } from "react-bootstrap";
import { AsyncTypeahead } from "react-bootstrap-typeahead";

interface Member {
  uuid: string;
  permissions: string[];
  display_name: string;
  picture_url?: string;
}

export default function Members({
  members,
  possiblePermissions,
  impliedPermissions,
  saveMemberPermissions,
  deleteMember,
}: {
  members: Member[];
  possiblePermissions?: string[];
  impliedPermissions?: string[][][];
  saveMemberPermissions: (
    prospectiveMember: boolean,
    uuid: string,
    selectedPermissions: string[]
  ) => Promise<any>;
  deleteMember: (uuid: string) => Promise<any>;
}) {
  const [prospectiveMembers, setProspectiveMembers] = useState<Member[]>([]);

  useEffect(() => {
    setProspectiveMembers(
      prospectiveMembers.filter((prospectiveMember) => {
        for (const member of members) {
          if (member.uuid === prospectiveMember.uuid) {
            return false;
          }
        }

        return true;
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
            impliedPermissions={impliedPermissions}
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
            impliedPermissions={impliedPermissions}
            saveMemberPermissions={saveMemberPermissions}
            deleteMember={deleteMember}
          />
        ))}

        {possiblePermissions ? (
          <MemberListInserter
            onInsert={(displayName, pictureUrl, userUuid) =>
              setProspectiveMembers([
                ...prospectiveMembers,
                {
                  uuid: userUuid,
                  display_name: displayName,
                  picture_url: pictureUrl,
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
  impliedPermissions,
  saveMemberPermissions,
  deleteMember,
}: {
  member: Member;
  prospectiveMember: boolean;
  possiblePermissions?: string[];
  impliedPermissions?: string[][][];
  saveMemberPermissions: (
    prospectiveMember: boolean,
    uuid: string,
    selectedPermissions: string[]
  ) => Promise<any>;
  deleteMember: (uuid: string) => Promise<any>;
}) {
  const [selectedPermissions, setSelectedPermissions] = useState(
    member.permissions || []
  );
  const auth = useAuth();
  const [deleting, setDeleting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(undefined);

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
      await deleteMember(member.uuid);
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
        show={deleting}
        onCancel={() => setDeleting(false)}
        onConfirm={() => doDelete()}
        username={member.display_name}
      />

      <ErrorModal error={error} onClose={() => setError(undefined)} />

      <tr>
        <td className="align-middle fit">
          <ProfilePicture src={member.picture_url} height="48px" width="48px" />
        </td>

        <td className="align-middle">
          <strong>
            <Link to={`/users/${member.uuid}`} className="text-decoration-none">
              {member.display_name}
            </Link>
          </strong>
          {auth?.getUserUuid() === member.uuid ? (
            <>
              <br />
              <em>(that's you!)</em>
            </>
          ) : (
            <></>
          )}
        </td>

        {possiblePermissions && member.permissions ? (
          <>
            <td className="align-middle">
              <RenderPermissions
                possiblePermissions={possiblePermissions}
                impliedPermissions={impliedPermissions}
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

interface MemberListInserterProps {
  existingMembers: Member[];
  onInsert: (
    username: string,
    user_uuid: string,
    picture_url: string | null
  ) => any;
}

interface SearchOption {
  user_uuid: string;
  display_name: string;
  picture_url: string | null;
}

function MemberListInserter({
  onInsert,
  existingMembers,
}: MemberListInserterProps) {
  const auth = useAuth();
  const searchRef = useRef(null);
  const [loading, setLoading] = useState(false);
  const [options, setOptions] = useState([]);
  const [error, setError] = useState("");

  if (!auth) {
    return <></>;
  }

  const handleSearch = async (query: string) => {
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
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (selected: SearchOption[]) => {
    onInsert(
      selected[0].display_name,
      selected[0].picture_url,
      selected[0].user_uuid
    );
    searchRef.current?.clear();
  };

  return (
    <tr>
      <td className="align-middle fit">
        <div
          className="d-flex align-items-center justify-content-center rounded-circle bg-default-profile-picture"
          style={{
            width: "48px",
            height: "48px",
            fontSize: "1rem",
          }}
        >
          <PersonPlusFill width="24px" height="24px" />
        </div>
      </td>

      <td className="align-middle">
        <AsyncTypeahead
          id="search-new-user"
          onSearch={handleSearch}
          filterBy={(option: SearchOption) => {
            for (const existing of existingMembers) {
              if (option.user_uuid === existing.uuid) {
                return false;
              }
            }

            return true;
          }}
          labelKey="display_name"
          options={options}
          isLoading={loading}
          placeholder="Search for User"
          onChange={handleChange}
          ref={searchRef}
          renderMenuItemChildren={(option: SearchOption) => (
            <>
              <ProfilePicture
                src={option.picture_url}
                height="24px"
                width="24px"
                className="me-2"
              />
              <span>{option.display_name}</span>
            </>
          )}
        />

        <div className="text-danger">{error}</div>
      </td>

      <td className="align-middle" />

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
  impliedPermissions,
  userUuid,
  onChange,
}: {
  possiblePermissions: string[];
  selectedPermissions: string[];
  impliedPermissions: string[][][];
  userUuid: string;
  onChange: (permissions: string[]) => any;
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

                for (const [a, b] of impliedPermissions) {
                  if (a[0] === permission) {
                    newUserPermissions.add(b[0]);
                  }
                }
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
